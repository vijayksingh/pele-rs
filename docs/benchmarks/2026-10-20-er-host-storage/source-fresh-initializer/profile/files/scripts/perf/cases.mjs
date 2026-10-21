// Public, deterministic synthetic workloads. No private corpus or implementation mirror.
export const revision = '458322a5b8abb1ae57ddf127c32017ab9fb14a92';
export const now = 1_704_067_200_000;
export const options = {limit:1_000_000,maxEdges:10_000,fontSize:16,padding:8,maxWidth:800,now};
export function cases() {
  const out=[];
  const add=(family,scale,n,source)=>out.push({id:`${family}/${scale}`,family,scale,units:n,source,options,resources:'fixed'});
  for(const[scale,n]of[['small',8],['medium',48],['large',200]]){
    add('flowchart',scale,n,'flowchart TB\n'+Array.from({length:n},(_,i)=>`n${i}[Node ${i}]`).join('\n')+'\n'+Array.from({length:n-1},(_,i)=>`n${i} --> n${i+1}`).join('\n'));
    add('sequence',scale,n,'sequenceDiagram\nparticipant App\nparticipant Core\n'+Array.from({length:n},(_,i)=>`${i%2?'Core-->>App':'App->>Core'}: Message ${i}`).join('\n'));
    add('er',scale,n,'erDiagram\n'+Array.from({length:n},(_,i)=>`E${i} {\n int id PK\n string label\n}`).join('\n')+'\n'+Array.from({length:n-1},(_,i)=>`E${i} ||--o{ E${i+1} : owns`).join('\n'));
    add('pie',scale,n,'pie title Benchmark\n'+Array.from({length:n},(_,i)=>`"Slice ${i}":${i+1}`).join('\n'));
    add('sankey',scale,n,'sankey-beta\n'+Array.from({length:n-1},(_,i)=>`N${i},N${i+1},${i%7+1}`).join('\n'));
    add('gantt',scale,n,'gantt\ndateFormat YYYY-MM-DD\naxisFormat %Y-%m-%d\ntodayMarker off\n'+Array.from({length:n},(_,i)=>`Task ${i}:t${i},2024-01-${String(i%28+1).padStart(2,'0')},${i%4+1}d`).join('\n'));
    const count=scale==='small'?3:scale==='medium'?15:60;
    add('railroad',scale,count,'railroad-beta\n'+Array.from({length:count},(_,i)=>`rule${i} = sequence(terminal("token ${i}"),choice(nonterminal("rule0"),optional(terminal("x"))),zeroOrMore(terminal(",")));`).join('\n'));
  }
  return out;
}
export const pairedStages = ['parse','parse-fresh-registry','parse-public','parse-public-fresh-registry','render-source','render-retained','render-mutable-core','render-mutable-publish'];
export const componentStages = ['registry','public-registry','model-json','graph-snapshot','graph-json-encode','graph-json-roundtrip','graph-binary-encode','graph-binary-decode','graph-apply','result-json'];
