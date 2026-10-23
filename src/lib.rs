//! Ordinary employee pension scenarios, not an entitlement determination.
//! Policy reference: Shanghai 2026-08-16 rules. Future rates are assumptions.
#[cfg(not(target_arch = "wasm32"))]
use chrono::Datelike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    pub current_age: u32,
    pub account_balance: f64,
    pub paid_months: u32,
    pub monthly_contribution: f64,
    pub use_maximum: bool,
    pub balance_includes_current_month: bool,
    pub employer_contribution_rate: f64,
    pub past_employer_contributions: Option<f64>,
    pub past_personal_contributions: Option<f64>,
    pub future_personal_monthly_cost: Option<f64>,
    pub valuation_month: String,
    pub birth_month: Option<String>,
    pub retirement_month: Option<String>,
    pub retirement_age: u32,
    pub statutory_retirement_month: Option<String>,
    pub minimum_months_override: Option<u32>,
    pub historical_average_index: f64,
    pub contribution_rate: f64,
    pub social_average_wage: f64,
    pub contribution_base_min: f64,
    pub contribution_base_max: f64,
    pub pension_base: f64,
    pub wage_growth_rate: f64,
    pub pension_base_growth_rate: f64,
    pub account_interest_rate: f64,
    pub payout_months_override: Option<u32>,
}

impl Default for Input {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let (year, month) = {
            let now = chrono::Utc::now() + chrono::Duration::hours(8);
            (now.year(), now.month())
        };
        #[cfg(target_arch = "wasm32")]
        let (year, month) = (2026, 10); // Browser supplies its current Beijing month.
        Self {
            current_age: 40,
            account_balance: 0.0,
            paid_months: 0,
            monthly_contribution: 1000.0,
            use_maximum: false,
            balance_includes_current_month: true,
            employer_contribution_rate: 0.16,
            past_employer_contributions: None,
            past_personal_contributions: None,
            future_personal_monthly_cost: None,
            valuation_month: format!("{:04}-{:02}", year, month),
            birth_month: None,
            retirement_month: None,
            retirement_age: 63,
            statutory_retirement_month: None,
            minimum_months_override: None,
            historical_average_index: 1.0,
            contribution_rate: 0.08,
            social_average_wage: 12577.0,
            contribution_base_min: 7546.0,
            contribution_base_max: 37731.0,
            pension_base: 12577.0,
            wage_growth_rate: 0.0,
            pension_base_growth_rate: 0.0,
            account_interest_rate: 0.0,
            payout_months_override: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Scenario {
    pub id: String,
    pub label: String,
    pub contribution_months: u32,
    pub total_paid_months: u32,
    pub claim_month: String,
    pub minimum_required_months: u32,
    pub eligible: bool,
    pub qualification_status: String,
    pub missing_months: u32,
    pub basic_pension: Option<f64>,
    pub account_pension: Option<f64>,
    pub monthly_total: Option<f64>,
    pub projected_account_balance: f64,
    pub average_index: f64,
    pub payout_months: Option<u32>,
    pub total_contribution_payback: Option<Recovery>,
    pub total_cost_is_estimate: bool,
    pub past_personal_cost_used: f64,
    pub past_personal_cost_estimated: bool,
    pub past_employer_cost_used: f64,
    pub past_employer_cost_estimated: bool,
    pub future_personal_cost: f64,
    pub future_employer_contributions: f64,
    pub simplified_recovery: Option<Recovery>,
    pub personal_payback: Option<Recovery>,
    pub future_account_principal: f64,
    pub notes: Vec<String>,
}

/// Static cumulative pension comparison, not investment return.
#[derive(Debug, Clone, Serialize)]
pub struct Recovery {
    pub cost: f64,
    pub payment_months: u64,
    pub years: u64,
    pub remaining_months: u64,
    pub age_years: u64,
    pub age_months: u64,
    pub beyond_100: bool,
}
fn recovery(cost: f64, monthly: Option<f64>, claim_age_months: u32) -> Option<Recovery> {
    let pension = monthly.filter(|p| *p > 0.0)?;
    let months = (cost / pension).ceil() as u64;
    let age = u64::from(claim_age_months) + months.saturating_sub(1);
    Some(Recovery {
        cost: money(cost),
        payment_months: months,
        years: months / 12,
        remaining_months: months % 12,
        age_years: age / 12,
        age_months: age % 12,
        beyond_100: age > 1200,
    })
}

#[derive(Debug, Serialize)]
pub struct Calculation {
    pub scenarios: Vec<Scenario>,
    pub warnings: Vec<String>,
    pub assumptions: Vec<String>,
    pub retirement_month: String,
    pub minimum_months_at_retirement: u32,
    pub effective_monthly_contribution: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Month(i32);
impl Month {
    fn parse(s: &str) -> Result<Self, String> {
        if s.len() != 7 || s.as_bytes()[4] != b'-' {
            return Err("年月必须采用 YYYY-MM 格式".into());
        }
        let y: i32 = s[..4].parse().map_err(|_| "年份无效")?;
        let m: i32 = s[5..].parse().map_err(|_| "月份无效")?;
        if !(1900..=2200).contains(&y) || !(1..=12).contains(&m) {
            return Err("年月超出有效范围（1900—2200 年）".into());
        }
        Ok(Self(y * 12 + m - 1))
    }
    fn add(self, n: u32) -> Self {
        Self(self.0 + n as i32)
    }
    fn year(self) -> i32 {
        self.0.div_euclid(12)
    }
    fn month(self) -> i32 {
        self.0.rem_euclid(12) + 1
    }
    fn label(self) -> String {
        format!("{:04}-{:02}", self.year(), self.month())
    }
}

/// Minimum contribution months for the selected retirement year.
pub fn minimum_months(year: i32) -> u32 {
    if year < 2030 {
        180
    } else {
        (180 + ((year - 2029) as u32) * 6).min(240)
    }
}

/// State Council 2005 table: whole retirement ages 40 through 70.
/// Fractional ages intentionally require explicit externally verified input.
pub fn payout_months(age: u32) -> Option<u32> {
    const TABLE: [u32; 31] = [
        233, 230, 226, 223, 220, 216, 212, 208, 204, 199, 195, 190, 185, 180, 175, 170, 164, 158,
        152, 145, 139, 132, 125, 117, 109, 101, 93, 84, 75, 65, 56,
    ];
    if (40..=70).contains(&age) {
        Some(TABLE[(age - 40) as usize])
    } else {
        None
    }
}

fn round4(x: f64) -> f64 {
    (x * 10000.0).round() / 10000.0
}
fn money(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
fn ceil_tenth(x: f64) -> f64 {
    ((x * 10.0 - 1e-9).ceil() / 10.0).max(0.0)
}

fn validate(i: &Input) -> Result<(), String> {
    for (name, value, max) in [
        (
            "历史单位实际缴费",
            i.past_employer_contributions,
            1_000_000_000.0,
        ),
        (
            "历史个人实际缴费",
            i.past_personal_contributions,
            100_000_000.0,
        ),
        (
            "未来每月自付成本",
            i.future_personal_monthly_cost,
            1_000_000.0,
        ),
    ] {
        if let Some(v) = value {
            if !v.is_finite() || !(0.0..=max).contains(&v) {
                return Err(format!("{name}须为 0 至 {max} 之间的有限数值"));
            }
        }
    }
    for (name, value, min, max) in [
        ("单位养老缴费比例", i.employer_contribution_rate, 0.0, 1.0),
        ("个人账户余额", i.account_balance, 0.0, 100_000_000.0),
        ("每月账户入账额", i.monthly_contribution, 0.0, 1_000_000.0),
        ("历史平均缴费指数", i.historical_average_index, 0.0, 10.0),
        ("个人缴费比例", i.contribution_rate, 0.001, 1.0),
        (
            "缴费指数参考月工资",
            i.social_average_wage,
            1.0,
            1_000_000.0,
        ),
        ("缴费基数下限", i.contribution_base_min, 1.0, 1_000_000.0),
        ("缴费基数上限", i.contribution_base_max, 1.0, 1_000_000.0),
        ("养老金计发基数", i.pension_base, 1.0, 1_000_000.0),
        ("缴费及工资年增长率", i.wage_growth_rate, -0.1, 0.2),
        ("计发基数年增长率", i.pension_base_growth_rate, -0.1, 0.2),
        ("账户年利率", i.account_interest_rate, 0.0, 0.2),
    ] {
        if !value.is_finite() || value < min || value > max {
            return Err(format!("{name}必须在 {min} 至 {max} 之间"));
        }
    }
    if !(16..=70).contains(&i.current_age) {
        return Err("当前年龄须为 16—70 岁".into());
    }
    if !(40..=70).contains(&i.retirement_age) {
        return Err("退休年龄假设须为 40—70 岁".into());
    }
    if i.paid_months > 660 {
        return Err("累计缴费月数不得超过 660".into());
    }
    if i.contribution_base_min > i.contribution_base_max {
        return Err("缴费基数下限不能高于上限".into());
    }
    if !i.use_maximum {
        let base = i.monthly_contribution / i.contribution_rate;
        if base + 0.01 < i.contribution_base_min || base - 0.01 > i.contribution_base_max {
            return Err(format!(
                "每月账户入账额对应基数 {:.2}，超出已设上下限 {:.2}—{:.2}；请调整缴费额或地区参数",
                base, i.contribution_base_min, i.contribution_base_max
            ));
        }
    }
    if let Some(n) = i.minimum_months_override {
        if !(180..=240).contains(&n) {
            return Err("核定最低缴费月数须为 180—240".into());
        }
    }
    if let Some(n) = i.payout_months_override {
        if !(1..=300).contains(&n) {
            return Err("手动计发月数须为 1—300".into());
        }
    }
    Ok(())
}

pub fn calculate(i: &Input) -> Result<Calculation, String> {
    validate(i)?;
    let start = Month::parse(&i.valuation_month)?;
    let birth = match i.birth_month.as_deref().filter(|s| !s.is_empty()) {
        Some(s) => Month::parse(s)?,
        None => Month(start.0 - i.current_age as i32 * 12),
    };
    let age_months = start.0 - birth.0;
    if !(16 * 12..=71 * 12 - 1).contains(&age_months) {
        return Err("出生年月与测算月不匹配，请检查年龄".into());
    }
    if i.birth_month.is_some() && age_months / 12 != i.current_age as i32 {
        return Err("出生年月与当前周岁不一致".into());
    }
    if i.paid_months as i32 > age_months - 16 * 12 {
        return Err("累计缴费月数超过自 16 周岁起的可用月份，请核对输入".into());
    }
    let retirement = match i.retirement_month.as_deref().filter(|s| !s.is_empty()) {
        Some(s) => Month::parse(s)?,
        None => birth.add(i.retirement_age * 12),
    };
    if retirement.0 < start.0 {
        return Err("退休年月不能早于当前测算月；本工具不处理已退休待遇重算".into());
    }
    // Snapshot includes valuation month by default; do not charge that month twice.
    let start = start.add(u32::from(i.balance_includes_current_month));
    let remaining = (retirement.0 - start.0 + 1) as u32;
    if remaining > 55 * 12 {
        return Err("退休时间跨度不得超过 55 年".into());
    }
    let retirement_age_months = retirement.0 - birth.0;
    if !(40 * 12..=70 * 12).contains(&retirement_age_months) {
        return Err("退休时年龄须在 40—70 岁范围内".into());
    }
    let factor = i.payout_months_override.or_else(|| {
        if retirement_age_months % 12 == 0 {
            payout_months((retirement_age_months / 12) as u32)
        } else {
            None
        }
    });
    let policy_year = match i
        .statutory_retirement_month
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        Some(s) => {
            let statutory = Month::parse(s)?;
            let statutory_age = statutory.0 - birth.0;
            if !(40 * 12..=70 * 12).contains(&statutory_age) {
                return Err("法定退休年月应对应本人 40—70 岁，请核对出生年月与法定退休年月".into());
            }
            statutory.year().min(retirement.year())
        }
        None => retirement.year(),
    };
    let min = i
        .minimum_months_override
        .unwrap_or_else(|| minimum_months(policy_year));
    let minimum_additional = min.saturating_sub(i.paid_months);
    let context = ScenarioContext {
        input: i,
        start,
        retirement,
        birth,
        minimum_months: min,
        payout_factor: factor,
    };
    let mut scenarios = vec![
        scenario(&context, 0, false, "stop_now", "现在停止缴费"),
        scenario(
            &context,
            minimum_additional,
            minimum_additional > remaining,
            "minimum",
            "继续按月缴至最低年限",
        ),
        scenario(
            &context,
            remaining,
            false,
            "retirement",
            "持续缴至所设退休月",
        ),
    ];
    for years in [5, 10, 15, 20] {
        if years * 12 <= remaining {
            scenarios.push(scenario(
                &context,
                years * 12,
                false,
                &format!("after_{years}"),
                &format!("再缴 {years} 年后停止"),
            ));
        }
    }
    let mut warnings = vec![
        "这是企业职工基本养老保险情景估算，不是社保机构核定待遇。".into(),
        "默认的退休年龄、历史指数和计发基数是可修改的示例假设，不代表已核定个人数据。养老金计发基数与缴费基数上限不是同一指标。".into(),
        "不包含视同缴费、过渡性养老金、特殊工种及特殊退休待遇；有此类情况不适用。".into(),
        "出生年月不能自动确定退休条件；请按本人性别、岗位类别及渐进式延迟退休政策填入核定退休年月。".into(),
        "最低年限优先使用核定覆盖值；否则取所选退休年与法定退休年中较早年份，未填法定年月时按所选退休年。弹性提前退休按实际退休年，弹性延迟按法定退休年，请正确填写。".into(),
    ];
    if factor.is_none() {
        warnings.push("非整周岁退休的计发月数需经办机构核实并手工填写；本工具不擅自插值，因此符合缴费年限的待遇仍显示待核定。".into());
    }
    if i.birth_month.is_none() {
        warnings.push(
            "未填出生年月：按当前年龄恰好为整周岁估算退休年月，可能有最多 11 个月误差。".into(),
        );
    }
    if minimum_additional > remaining {
        warnings.push("补足最低年限将超过所设退休月，涉及延长缴费资格及当时年限要求，不能视作允许一次性补缴或已具备领取资格。".into());
    }
    if i.paid_months > 0 && i.historical_average_index == 0.0 {
        return Err("已有缴费月份时，历史平均缴费指数必须大于 0".into());
    }
    let effective = if i.use_maximum {
        i.contribution_base_max * i.contribution_rate
    } else {
        i.monthly_contribution
    };
    Ok(Calculation {
        scenarios, warnings, retirement_month: retirement.label(),
        minimum_months_at_retirement: min, effective_monthly_contribution: money(effective),
        assumptions: vec![
            "个人＋单位总缴费回本是总用工成本的静态名义比较，不表示单位缴费可一比一转为工资，也不是养老体系偿付或投资收益率。默认单位费率 16% 是可调假设，未来单位费按实际缴费基数乘该比例，非全额工资。".into(),
            "总缴费历史成本：已填个人/单位实际额优先；缺失个人额以当前余额代理，缺失单位额优先以历史个人实缴÷当前个人费率×单位费率代理；个人实缴也缺失时才改用当前余额。任一缺失均显著标为历史估算；余额含历史利息且历史费率可能变化，不能视作实际历史费总。".into(),
            "灵活就业若未来个人自付已含全部费用，单位费率应设为 0，历史单位实缴填 0，避免重复计算。".into(),
            "退休当月仍缴费，假设手续审核通过后次月起领；计发月数按退休当月年龄，不按次月年龄。实际审核或补发时间需经办核定。".into(),
            "简化余额回收：以当前含历史利息的账户余额加未来入账本金为比较额，不把未来利息算作投入；这不是实际自付缴费回本。".into(),
            "自付缴费回本仅在填写历史实际个人缴费后计算；未来月自付默认等于月入账额，仅适用于普通单位职工，灵活就业应填写全部自付成本。未来自付与入账额按同一缴费增长假设变化。".into(),
            "回收年数＝比较额÷退休初始基础及账户养老金合计，向上取整到领取月数；养老金次月开始领取为第 1 期，第 N 期覆盖成本的月份＝领取起始月加 N−1 个月；零成本记为起领月。零养老金或待核定不适用。忽略退休后调整、折现、机会成本及身故待遇，不是投资收益率或寿命预测；超过 100 岁仍展示机械结果。".into(),
            "余额和已缴月数默认均已含计算起始月；未来从次月缴费且包含退休当月。取消勾选表示余额及已缴月数截至上月，从计算起始月开始。未缴月份不进入平均指数分母。".into(),
            "未来账户入账额、缴费基数上下限及指数参考工资按同一工资增长率逐年变化；默认 0 表示一直按当前缴费额，勾选上限则按设定上限缴费。".into(),
            "每月账户入账额指每月计入个人账户的缴费本金，不含记账利息，不是单位与个人缴费合计。仅标准单位职工用入账额÷8%推算缴费基数，非标准口径应另行核实。".into(),
            "历史平均指数必须来自缴费记录或自行明确假设，不从个人账户余额反推。未来月指数保留 4 位小数，与历史月份加权。".into(),
            "账户利息为可调情景：年内按月累计简单利息、年末入账，领取时计入当年累计模拟利息；不等同经办机构的历史记账利息复算。".into(),
            "停止缴费不等于立即领取。除需延缴待核定的情景外，均比较在所设退休次月开始领取的每月待遇（假设退休当月审核通过），停缴后的账户继续按假设利率计息。".into(),
            "2026 上海规则：缴费年限折算年后保留 4 位小数，基础及个人账户养老金分别见分进角；结果为退休时名义金额，未作购买力折现或退休后调整。".into(),
        ],
    })
}

#[derive(Clone, Copy)]
struct ScenarioContext<'a> {
    input: &'a Input,
    start: Month,
    retirement: Month,
    birth: Month,
    minimum_months: u32,
    payout_factor: Option<u32>,
}

fn scenario(
    context: &ScenarioContext<'_>,
    contributions: u32,
    extension: bool,
    id: &str,
    label: &str,
) -> Scenario {
    let ScenarioContext {
        input: i,
        start,
        retirement,
        birth,
        minimum_months: min,
        payout_factor: factor,
    } = *context;
    let months_to_retirement = (retirement.0 - start.0 + 1) as u32;
    let horizon = months_to_retirement.max(contributions);
    let claim = start.add(horizon);
    let total = i.paid_months + contributions;
    let missing = min.saturating_sub(total);
    let eligible = missing == 0 && !extension && factor.is_some();
    let status = if missing > 0 {
        "insufficient"
    } else if extension || factor.is_none() {
        "needs_confirmation"
    } else {
        "eligible"
    };
    let first_payment = if i.use_maximum {
        i.contribution_base_max * i.contribution_rate
    } else {
        i.monthly_contribution
    };
    let mut future_principal = 0.0;
    let mut future_cost = 0.0;
    let mut future_employer = 0.0;
    let mut balance = i.account_balance;
    let mut year_principal = balance;
    let mut new_principal = 0.0;
    let mut accrued_interest = 0.0;
    let mut index_sum = i.historical_average_index * i.paid_months as f64;
    for m in 0..horizon {
        let month = start.add(m);
        let growth = (1.0 + i.wage_growth_rate).powi((m / 12) as i32);
        if m < contributions {
            let payment = first_payment * growth;
            future_principal += payment;
            future_employer += payment / i.contribution_rate * i.employer_contribution_rate;
            future_cost += i.future_personal_monthly_cost.unwrap_or(first_payment) * growth;
            balance += payment;
            new_principal += payment;
            index_sum += round4((payment / i.contribution_rate) / (i.social_average_wage * growth));
        }
        accrued_interest += (year_principal + new_principal) * i.account_interest_rate / 12.0;
        if month.month() == 12 {
            balance += accrued_interest;
            accrued_interest = 0.0;
            new_principal = 0.0;
            year_principal = balance;
        }
    }
    balance += accrued_interest;
    let average = if total > 0 {
        index_sum / total as f64
    } else {
        0.0
    };
    let base = i.pension_base * (1.0 + i.pension_base_growth_rate).powf(horizon as f64 / 12.0);
    let (basic, account, monthly) = if missing > 0 {
        (Some(0.0), Some(0.0), Some(0.0))
    } else if eligible {
        let basic = ceil_tenth(base * (1.0 + average) / 2.0 * round4(total as f64 / 12.0) * 0.01);
        let account = ceil_tenth(balance / factor.unwrap() as f64);
        (Some(basic), Some(account), Some(money(basic + account)))
    } else {
        (None, None, None)
    };
    let mut notes = vec![];
    if missing > 0 {
        notes.push(format!("按所设退休年份最低 {min} 个月要求，尚缺 {missing} 个月，故月领显示为 0；账户余额并不归零。"));
    }
    if extension {
        notes.push(format!("需再缴 {contributions} 个月，超过所设退休月；{} 仅是补到该退休年份 {min} 个月门槛的数学日期，不代表该日可退休或届时仍适用此门槛。请向经办机构核定延缴资格和年限。", claim.label()));
    }
    if factor.is_none() {
        notes.push("缺少经核定的非整周岁退休计发月数。".into());
    }
    if contributions == 0 {
        notes.push("不再新增缴费，但到退休前个人账户按所设利率继续模拟计息。".into());
    }
    let history_personal = i.past_personal_contributions.unwrap_or(i.account_balance);
    let history_employer = i
        .past_employer_contributions
        .unwrap_or(history_personal / i.contribution_rate * i.employer_contribution_rate);
    Scenario {
        id: id.into(),
        label: label.into(),
        contribution_months: contributions,
        total_paid_months: total,
        claim_month: claim.label(),
        minimum_required_months: min,
        eligible,
        qualification_status: status.into(),
        missing_months: missing,
        basic_pension: basic,
        account_pension: account,
        monthly_total: monthly,
        projected_account_balance: money(balance),
        average_index: round4(average),
        payout_months: if extension { None } else { factor },
        total_contribution_payback: recovery(
            history_personal + history_employer + future_cost + future_employer,
            monthly,
            (claim.0 - birth.0) as u32,
        ),
        past_personal_cost_used: money(history_personal),
        past_personal_cost_estimated: i.past_personal_contributions.is_none(),
        past_employer_cost_used: money(history_employer),
        past_employer_cost_estimated: i.past_employer_contributions.is_none(),
        future_personal_cost: money(future_cost),
        total_cost_is_estimate: i.past_personal_contributions.is_none()
            || i.past_employer_contributions.is_none(),
        future_employer_contributions: money(future_employer),
        simplified_recovery: recovery(
            i.account_balance + future_principal,
            monthly,
            (claim.0 - birth.0) as u32,
        ),
        personal_payback: i
            .past_personal_contributions
            .and_then(|past| recovery(past + future_cost, monthly, (claim.0 - birth.0) as u32)),
        future_account_principal: money(future_principal),
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> Input {
        Input {
            current_age: 40,
            account_balance: 100_000.0,
            paid_months: 240,
            monthly_contribution: 1006.16,
            valuation_month: "2026-10".into(),
            ..Input::default()
        }
    }
    #[test]
    fn increasing_minimum_schedule() {
        assert_eq!(minimum_months(2029), 180);
        assert_eq!(minimum_months(2030), 186);
        assert_eq!(minimum_months(2038), 234);
        assert_eq!(minimum_months(2039), 240);
        assert_eq!(minimum_months(2100), 240);
    }
    #[test]
    fn national_factors() {
        assert_eq!(payout_months(60), Some(139));
        assert_eq!(payout_months(63), Some(117));
        assert_eq!(payout_months(70), Some(56));
        assert_eq!(payout_months(71), None);
    }
    #[test]
    fn static_known_example() {
        let r = calculate(&input()).unwrap();
        let s = &r.scenarios[0];
        assert_eq!(s.basic_pension, Some(2515.4));
        assert_eq!(s.account_pension, Some(854.8));
        assert_eq!(s.monthly_total, Some(3370.2));
        assert_eq!(s.projected_account_balance, 100_000.0);
    }
    #[test]
    fn stop_below_minimum_is_zero() {
        let mut i = input();
        i.paid_months = 120;
        let r = calculate(&i).unwrap();
        assert_eq!(r.scenarios[0].monthly_total, Some(0.0));
        assert_eq!(r.scenarios[0].missing_months, 120);
        assert_eq!(r.scenarios[1].total_paid_months, 240);
    }
    #[test]
    fn scenarios_stop_at_retirement() {
        let mut i = input();
        i.current_age = 60;
        let r = calculate(&i).unwrap();
        assert_eq!(r.scenarios.len(), 3);
        assert_eq!(r.scenarios[2].contribution_months, 36);
    }
    #[test]
    fn cannot_claim_early() {
        let r = calculate(&input()).unwrap();
        for s in r.scenarios {
            assert_eq!(s.claim_month, "2049-11");
        }
    }
    #[test]
    fn minimum_after_retirement_is_unconfirmed() {
        let mut i = input();
        i.current_age = 62;
        i.paid_months = 12;
        let r = calculate(&i).unwrap();
        assert_eq!(r.scenarios[1].qualification_status, "needs_confirmation");
        assert_eq!(r.scenarios[1].monthly_total, None);
    }
    #[test]
    fn fractional_age_needs_factor() {
        let mut i = input();
        i.birth_month = Some("1986-04".into());
        i.retirement_month = Some("2049-10".into());
        let r = calculate(&i).unwrap();
        assert_eq!(r.scenarios[0].monthly_total, None);
        i.payout_months_override = Some(117);
        assert!(calculate(&i).unwrap().scenarios[0].eligible);
    }
    #[test]
    fn maximum_is_personal_portion() {
        let mut i = input();
        i.use_maximum = true;
        let r = calculate(&i).unwrap();
        assert_eq!(r.effective_monthly_contribution, 3018.48);
    }
    #[test]
    fn invalid_inputs_rejected() {
        let mut i = input();
        i.account_balance = -1.0;
        assert!(calculate(&i).is_err());
        i = input();
        i.monthly_contribution = 1.0;
        assert!(calculate(&i).is_err());
        i = input();
        i.valuation_month = "2026-13".into();
        assert!(calculate(&i).is_err());
    }
    #[test]
    fn birth_age_mismatch_rejected() {
        let mut i = input();
        i.birth_month = Some("1990-01".into());
        assert!(calculate(&i).is_err());
    }
    #[test]
    fn interest_continues_after_stopping() {
        let mut i = input();
        i.account_interest_rate = 0.03;
        let r = calculate(&i).unwrap();
        assert!(r.scenarios[0].projected_account_balance > 100_000.0);
    }
    #[test]
    fn years_round_to_four_decimals() {
        let mut i = input();
        i.paid_months = 241;
        let s = calculate(&i).unwrap().scenarios.remove(0);
        assert_eq!(s.basic_pension, Some(2525.9));
    }
    #[test]
    fn zero_history_excludes_history_index() {
        let mut i = input();
        i.paid_months = 0;
        i.account_balance = 0.0;
        i.historical_average_index = 0.0;
        let r = calculate(&i).unwrap();
        assert_eq!(r.scenarios[2].average_index, 1.0);
    }
    #[test]
    fn historical_and_future_indices_weight_months() {
        let mut i = input();
        i.current_age = 43;
        i.historical_average_index = 2.0;
        let r = calculate(&i).unwrap();
        assert_eq!(r.scenarios[2].total_paid_months, 480);
        assert_eq!(r.scenarios[2].average_index, 1.5);
    }
    #[test]
    fn early_retirement_uses_earlier_actual_year() {
        let mut i = input();
        i.current_age = 59;
        i.birth_month = Some("1967-10".into());
        i.retirement_month = Some("2029-10".into());
        i.statutory_retirement_month = Some("2030-10".into());
        assert_eq!(calculate(&i).unwrap().minimum_months_at_retirement, 180);
    }
    #[test]
    fn delayed_retirement_locks_statutory_year() {
        let mut i = input();
        i.current_age = 59;
        i.birth_month = Some("1967-10".into());
        i.retirement_month = Some("2030-10".into());
        i.statutory_retirement_month = Some("2029-10".into());
        assert_eq!(calculate(&i).unwrap().minimum_months_at_retirement, 180);
        i.minimum_months_override = Some(186);
        assert_eq!(calculate(&i).unwrap().minimum_months_at_retirement, 186);
    }

    #[test]
    fn impossible_statutory_date_is_rejected() {
        let mut i = input();
        i.statutory_retirement_month = Some("1900-01".into());
        assert!(calculate(&i).is_err());
    }
    #[test]
    fn recovery_hand_calculation() {
        // Ceiling 30 payments; claim begins one month after retirement at 63.
        let s = calculate(&input()).unwrap().scenarios.remove(0);
        let r = s.simplified_recovery.unwrap();
        assert_eq!((r.payment_months, r.age_years, r.age_months), (30, 65, 6));
        assert!(s.personal_payback.is_none());
        let r = recovery(12000.0, Some(1000.0), 720).unwrap();
        assert_eq!((r.payment_months, r.age_years, r.age_months), (12, 60, 11));
    }
    #[test]
    fn recovery_principal_excludes_future_interest() {
        let mut i = input();
        i.account_interest_rate = 0.05;
        i.past_personal_contributions = Some(80000.0);
        i.future_personal_monthly_cost = Some(2000.0);
        let s = calculate(&i).unwrap().scenarios.remove(3); // Five years
        assert_eq!(s.future_account_principal, 60369.6);
        assert_eq!(s.simplified_recovery.unwrap().cost, 160369.6);
        assert_eq!(s.personal_payback.unwrap().cost, 200000.0);
        assert!(s.projected_account_balance > 160369.6);
    }
    #[test]
    fn recovery_zero_pending_and_long_life() {
        assert!(recovery(100.0, Some(0.0), 720).is_none());
        assert!(recovery(100.0, None, 720).is_none());
        let r = recovery(1000000.0, Some(1000.0), 720).unwrap();
        assert_eq!((r.age_years, r.age_months), (143, 3));
        assert!(r.beyond_100);
        assert_eq!(recovery(0.0, Some(1000.0), 720).unwrap().payment_months, 0);
        let mut i = input();
        i.paid_months = 120;
        assert!(calculate(&i).unwrap().scenarios[0]
            .simplified_recovery
            .is_none());
        i = input();
        i.current_age = 62;
        i.paid_months = 12;
        assert!(calculate(&i).unwrap().scenarios[1]
            .simplified_recovery
            .is_none());
    }
    #[test]
    fn recovery_personal_cost_default_and_growth() {
        let mut i = input();
        i.past_personal_contributions = Some(0.0);
        i.wage_growth_rate = 0.1;
        let s = calculate(&i).unwrap().scenarios.remove(3);
        assert_eq!(s.personal_payback.unwrap().cost, s.future_account_principal);
        i.future_personal_monthly_cost = Some(-1.0);
        assert!(calculate(&i).is_err());
    }
    #[test]
    fn employer_history_estimate_and_actual() {
        let i = input();
        let s = calculate(&i).unwrap().scenarios.remove(3);
        assert!(s.total_cost_is_estimate);
        assert_eq!(s.future_employer_contributions, 120739.2);
        assert_eq!(s.total_contribution_payback.unwrap().cost, 481108.8);
        let mut i = i;
        i.past_personal_contributions = Some(80000.0);
        i.past_employer_contributions = Some(150000.0);
        let s = calculate(&i).unwrap().scenarios.remove(3);
        assert!(!s.total_cost_is_estimate);
        assert_eq!(s.total_contribution_payback.unwrap().cost, 411108.8);
    }
    #[test]
    fn snapshot_and_retirement_boundary() {
        let mut i = input();
        i.current_age = 63;
        let r = calculate(&i).unwrap();
        let s = &r.scenarios[2];
        assert_eq!(s.contribution_months, 0);
        assert_eq!(s.claim_month, "2026-11");
        assert_eq!(s.payout_months, Some(117));
        i.balance_includes_current_month = false;
        let r = calculate(&i).unwrap();
        let s = &r.scenarios[2];
        assert_eq!(s.contribution_months, 1);
        assert_eq!(s.future_account_principal, 1006.16);
        assert_eq!(s.claim_month, "2026-11");
        assert_eq!(s.payout_months, Some(117));
    }
    #[test]
    fn employer_actual_zero_and_invalid() {
        let mut i = input();
        i.employer_contribution_rate = 0.0;
        i.past_personal_contributions = Some(80000.0);
        i.past_employer_contributions = Some(0.0);
        i.future_personal_monthly_cost = Some(2500.0);
        let s = calculate(&i).unwrap().scenarios.remove(3);
        assert_eq!(s.future_employer_contributions, 0.0);
        assert_eq!(
            s.total_contribution_payback.unwrap().cost,
            s.personal_payback.unwrap().cost
        );
        i.employer_contribution_rate = -0.1;
        assert!(calculate(&i).is_err());
    }
    #[test]
    fn recovery_first_and_twelfth_payment_months() {
        let one = recovery(1000.0, Some(1000.0), 721).unwrap();
        assert_eq!(
            (one.payment_months, one.age_years, one.age_months),
            (1, 60, 1)
        );
        let twelve = recovery(12000.0, Some(1000.0), 721).unwrap();
        assert_eq!(
            (twelve.payment_months, twelve.age_years, twelve.age_months),
            (12, 61, 0)
        );
        let zero = recovery(0.0, Some(1000.0), 721).unwrap();
        assert_eq!(zero.age_months, 1);
    }
    #[test]
    fn historical_employer_estimate_prefers_known_personal_principal() {
        let mut i = input();
        i.past_personal_contributions = Some(80000.0);
        let s = calculate(&i).unwrap().scenarios.remove(0);
        assert_eq!(s.past_employer_cost_used, 160000.0);
        assert_eq!(s.total_contribution_payback.unwrap().cost, 240000.0);
        assert!(s.total_cost_is_estimate);
        assert!(!s.past_personal_cost_estimated);
    }
}
