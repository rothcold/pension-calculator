const fs=require('fs'), assert=require('assert');
(async()=>{
 const {instance:{exports:w}}=await WebAssembly.instantiate(fs.readFileSync('dist/pension_browser.wasm'),{});
 function decode(n){let p=Number(n&0xffffffffn),l=Number(n>>32n);try{return JSON.parse(new TextDecoder().decode(new Uint8Array(w.memory.buffer,p,l)))}finally{w.release(p,l)}}
 function calc(i){let b=new TextEncoder().encode(JSON.stringify(i)),p=w.allocate(b.length);try{new Uint8Array(w.memory.buffer,p,b.length).set(b);return decode(w.calculate(p,b.length));}finally{w.release(p,b.length)}}
 const d=decode(w.defaults());Object.assign(d,{current_age:40,account_balance:100000,paid_months:240,monthly_contribution:1006.16,valuation_month:'2026-10'});
 let r=calc(d);assert.equal(r.scenarios[0].monthly_total,3370.2);assert.equal(r.scenarios[0].basic_pension,2515.4);
 let recovery=r.scenarios[0].simplified_recovery;assert.equal(recovery.payment_months,30);assert.equal(recovery.age_years,65);assert.equal(recovery.age_months,6);assert.equal(r.scenarios[0].personal_payback,null);
 let five=calc({...d,account_interest_rate:0.05,past_personal_contributions:80000,future_personal_monthly_cost:2000}).scenarios[3];assert.equal(five.simplified_recovery.cost,160369.6);assert.equal(five.personal_payback.cost,200000);
 r=calc({...d,paid_months:120});assert.equal(r.scenarios[0].simplified_recovery,null);assert.equal(r.scenarios[0].monthly_total,0);assert.equal(r.scenarios[0].missing_months,120);
 assert.equal(calc({...d,use_maximum:true}).effective_monthly_contribution,3018.48);
 assert.equal(calc({...d,current_age:60}).scenarios.length,3);
 assert.equal(calc({...d,birth_month:'1986-04',retirement_month:'2049-10'}).scenarios[0].monthly_total,null);
 assert.ok(calc({...d,account_balance:-1}).error);
 for(let i=0;i<1000;i++)assert.equal(calc(d).scenarios[0].monthly_total,3370.2);
 let total=calc(d).scenarios[3];assert.equal(total.total_cost_is_estimate,true);assert.equal(total.total_contribution_payback.cost,481108.8);assert.equal(total.claim_month,'2049-11');
 total=calc({...d,past_personal_contributions:80000,past_employer_contributions:150000}).scenarios[3];assert.equal(total.total_cost_is_estimate,false);assert.equal(total.total_contribution_payback.cost,411108.8);
 let boundary=calc({...d,current_age:63});assert.equal(boundary.scenarios[2].contribution_months,0);assert.equal(boundary.scenarios[2].claim_month,'2026-11');assert.equal(boundary.scenarios[2].payout_months,117);
 boundary=calc({...d,current_age:63,balance_includes_current_month:false});assert.equal(boundary.scenarios[2].contribution_months,1);assert.equal(boundary.scenarios[2].future_account_principal,1006.16);
 console.log('WASM numeric, validation, fractional-age, 1000 repeated calls: PASS');
 const html=fs.readFileSync('dist/index.html','utf8');new Function(html.match(/<script>([\s\S]*)<\/script>/)[1]);
 assert(!html.includes("fetch('/api/"));console.log('UI JS syntax and no network calculation API: PASS');
})();
