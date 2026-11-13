const fs=require('fs'), assert=require('assert'), vm=require('vm');
function checkInlineScripts(html, filename){
 const scripts=[...html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script\s*>/gi)]
  .filter(match=>!/(?:^|\s)src(?:\s|=|$)/i.test(match[1])).map(match=>match[2]);
 for(const [index, script] of scripts.entries())new vm.Script(script,{filename:filename+':inline-'+(index+1)});
 return scripts.length;
}
const fixture='<script>const first=1;</script><p>Between scripts</p><script defer src="/external.js"></script><script>const second=2;</script>';
assert.equal(checkInlineScripts(fixture,'multiple scripts'),2);
assert.equal(checkInlineScripts('<script nonce="test" data-src="label">const value=1;</script>','attributes'),1);
assert.equal(checkInlineScripts('<SCRIPT type="text/javascript">let value=1;</SCRIPT>','case'),1);
assert.equal(checkInlineScripts('<script src="/external.js">not inline JavaScript</script>','external'),0);
assert.equal(checkInlineScripts('<p>No scripts</p>','empty'),0);
assert.throws(()=>checkInlineScripts(fixture.replace('const first=1;','const =;'),'invalid first'),SyntaxError);
assert.throws(()=>checkInlineScripts(fixture.replace('const second=2;','const =;'),'invalid last'),SyntaxError);
assert.throws(()=>checkInlineScripts('<script>const shared=1;</script><script>const =;</script><script>const final=2;</script>','invalid middle'),SyntaxError);
console.log('Inline script extraction regressions: PASS');

const origin='https://pension-calculator.rothcold.me/';
const read=path=>fs.readFileSync(path,'utf8');
const web=read('web/index.html'), http=read('frontend/index.html'), built=read('dist/index.html');
const section=(html,id)=>{
 const match=html.match(new RegExp('<section[^>]* id="'+id+'"[^>]*>([\\s\\S]*?)</section>'));
 assert(match,'Missing static section: '+id);
 assert(!/\bhidden\b|<script\b|<details\b/.test(match[0]),'Explanation must be visible without JavaScript');
 return match[1];
};
for(const [path,html] of [['web/index.html',web],['frontend/index.html',http],['dist/index.html',built]]){
 assert.equal(checkInlineScripts(html,path),2,path+': validate both inline scripts, skip external scripts');
 const head=html.match(/<head>([\s\S]*?)<\/head>/)[1];
 const title=head.match(/<title>(.*?)<\/title>/)[1];
 assert.equal(title,'上海养老金计算器｜普通企业职工情景估算');
 const meta=(attribute,key)=>{
  const matches=[...head.matchAll(new RegExp('<meta '+attribute+'="'+key+'" content="([^"]*)">','g'))];
  assert.equal(matches.length,1,path+': '+key);
  return matches[0][1];
 };
 const description=meta('name','description');
 for(const text of ['上海普通企业职工','示例','不替代社保经办核定'])assert(description.includes(text));
 assert.equal(meta('property','og:title'),title);
 assert.equal(meta('property','og:description'),description);
 assert.equal(meta('property','og:type'),'website');
 assert.equal(meta('property','og:locale'),'zh_CN');
 assert.equal(meta('property','og:site_name'),'上海养老金计算器');
 assert.equal(meta('property','og:url'),origin);
 assert.deepEqual([...head.matchAll(/<link rel="canonical" href="([^"]*)">/g)].map(m=>m[1]),[origin]);
 assert(!/noindex/i.test(head));
 const scope=section(html,'scope'), privacy=section(html,'privacy');
 for(const text of ['上海普通企业职工','城乡居民','机关事业单位','视同缴费','过渡性养老金','特殊工种','提前退休','跨地区','40 岁','1,000 元','63 岁','12,577 元','示例','假设'])assert(scope.includes(text),path+': '+text);
 for(const text of ['Vercel Analytics','Speed Insights','访问与性能数据','不构成对第三方脚本采集或传输行为的保证'])assert(privacy.includes(text),path+': '+text);
 for(const id of ['scope','privacy'])assert(html.includes('href="#'+id+'"'));
 assert(scope.includes('href="https://github.com/rothcold/pension-calculator/blob/main/README.md"'));
 for(const service of ['analytics','speed-insights'])assert(privacy.includes('href="https://vercel.com/docs/'+service+'/privacy-policy"'));
 for(const script of ['insights','speed-insights'])assert(head.includes('src="/_vercel/'+script+'/script.js"'));
 assert(!/无第三方脚本|不加载第三方脚本|无埋点|没有分析埋点|不接第三方分析服务|不会保存或上传至第三方|账户输入不上传/.test(html));
}
assert.equal(section(web,'scope'),section(http,'scope'));
assert(section(web,'privacy').includes('浏览器内的 Rust / WebAssembly'));
assert(section(http,'privacy').includes('当前站点的 /api/calculate'));
const sharedBody=html=>html.match(/<body>([\s\S]*?)<\/main>/)[1]
 .replace(/<span class="badge">.*?<\/span>/,'')
 .replace(/<span id="status" role="status">.*?<\/span>/,'')
 .replace(/(<h2 id="privacy-title">.*?<\/h2>\s*)<p>.*?<\/p>/,'$1');
assert.equal(sharedBody(web),sharedBody(http),'Keep shared UI copy identical');
assert(!web.includes("fetch('/api/"),'WASM must not call the calculation API');
assert(http.includes("fetch('/api/calculate'"),'HTTP calculation transport must remain');
assert(http.includes("fetch('/api/defaults'"),'HTTP defaults transport must remain');
assert.equal(built,web,'Build must copy the current static page');
for(const file of ['robots.txt','sitemap.xml'])assert.equal(read('dist/'+file),read('web/'+file));
assert.equal(read('dist/robots.txt'),'User-agent: *\nAllow: /\n\nSitemap: '+origin+'sitemap.xml\n');
const sitemap=read('dist/sitemap.xml');
assert(sitemap.includes('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'));
assert.deepEqual([...sitemap.matchAll(/<loc>(.*?)<\/loc>/g)].map(m=>m[1]),[origin]);
for(const file of ['favicon.svg','favicon-32.png','favicon.ico'])assert(fs.readFileSync('dist/'+file).equals(fs.readFileSync('frontend/'+file)));
const readme=read('README.md');
for(const text of ['Vercel Analytics','Speed Insights','/api/calculate','浏览器内','不构成对第三方脚本采集或传输行为的保证'])assert(readme.includes(text));
assert(!/不加载第三方脚本|没有分析埋点|不接第三方分析服务/.test(readme));
console.log('Frontend script syntax, shared copy, privacy, SEO and build assets: PASS');
