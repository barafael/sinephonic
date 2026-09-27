const FL=['C','D♭','D','E♭','E','F','G♭','G','A♭','A','B♭','B'];
const SH=['C','C♯','D','D♯','E','F','F♯','G','G♯','A','A♯','B'];
const R5=[[1,1],[16,15],[9,8],[6,5],[5,4],[4,3],[45,32],[3,2],[8,5],[5,3],[9,5],[15,8]];
const R7=R5.map((r,i)=>i===6?[7,5]:i===10?[7,4]:r);
const IVS=['P1','m2','M2','m3','M3','P4','TT','P5','m6','M6','m7','M7'];
const IVC={12:'P8',13:'m9',14:'M9',15:'m10',16:'M10',17:'P11',18:'A11',19:'P12',20:'m13',21:'M13',22:'m14',23:'M14',24:'P15'};
const BRIGHT=[0,-1.6,0.4,-1.2,1.3,-0.2,0.8,0.1,-1.3,0.9,-0.5,1.2];
const gcd=(a,b)=>{while(b){[a,b]=[b,a%b]}return a};
const lcm=(a,b)=>a/gcd(a,b)*b;
const clamp=(x,a=0,b=1)=>Math.max(a,Math.min(b,x));
const ACC='oklch(0.56 0.16 35)';const col=pc=>'#1d1b18';
const isBlack=pc=>[1,3,6,8,10].includes(pc);
const ratioFor=(d,lim)=>{let [p,q]=(lim==='7-limit'?R7:R5)[d%12];p*=2**Math.floor(d/12);const g=gcd(p,q);return [p/g,q/g]};
const ivName=d=>d<12?IVS[d]:(IVC[d]||IVS[d%12]+'+'+Math.floor(d/12)+'oct');
const PRESETS=[['C–G–A♭–E♭',[48,55,56,63]],['C7♯9',[48,52,55,58,63]],['C7♯9♯5',[48,52,56,58,63]],['Cmaj7',[48,52,55,59]],['C major',[48,52,55]],['C minor',[48,51,55]],['C7♭9',[48,52,55,58,61]],['Quartal',[50,55,60,65]],['Cluster',[60,61,62]]];
const REFS=[['maj',[48,52,55]],['min',[48,51,55]],['maj7',[48,52,55,59]],['7♯9',[48,52,55,58,63]],['7♯9♯5',[48,52,56,58,63]],['dim7',[48,51,54,57]],['cluster',[60,61,62]],['quartal',[50,55,60,65]]];

function diss(f1,f2,a1,a2){const fm=Math.min(f1,f2),df=Math.abs(f2-f1),s=0.24/(0.0207*fm+18.96);return Math.min(a1,a2)*(Math.exp(-3.51*s*df)-Math.exp(-5.75*s*df))}
function partials(f,t){return t==='sine'?[[f,1]]:[1,2,3,4,5,6].map(k=>[f*k,0.88**(k-1)])}
function pairRough(f1,f2,t){let s=0;for(const [a,x] of partials(f1,t))for(const [b,y] of partials(f2,t))s+=diss(a,b,x,y);return s}

function reading(root,pcs,bassPc,N){
  const rel=new Set(pcs.map(p=>(p-root+12)%12)),h=i=>rel.has(i);
  const P5=h(7),hasSev=h(11)||h(10);
  const third=h(4)?'M':h(3)?'m':h(5)?'sus4':(!hasSev&&h(2))?'sus2':'no3';
  const deg={0:'R'};
  if(third==='M')deg[4]='3';if(third==='m')deg[3]='♭3';if(third==='sus4')deg[5]='4';if(third==='sus2')deg[2]='2';
  const dim=third==='m'&&h(6)&&!P5, aug=third==='M'&&h(8)&&!P5;
  if(P5)deg[7]='5';if(dim)deg[6]='♭5';if(aug)deg[8]='♯5';
  let sev=null;
  if(h(11)){sev='maj';deg[11]='7'}else if(h(10)){sev='dom';deg[10]='♭7'}else if(dim&&h(9)){sev='dim';deg[9]='♭♭7'}
  const nat=[],alts=[];
  if(h(1)){deg[1]='♭9';alts.push('♭9')}
  if(h(2)&&deg[2]==null){deg[2]='9';nat.push(9)}
  if(h(3)&&deg[3]==null){deg[3]='♯9';alts.push('♯9')}
  if(h(5)&&deg[5]==null){deg[5]='11';nat.push(11)}
  if(h(6)&&deg[6]==null){if(P5||sev){deg[6]='♯11';alts.push('♯11')}else{deg[6]='♭5';alts.push('♭5')}}
  if(h(8)&&deg[8]==null){if(sev){deg[8]='♭13';alts.push('♭13')}else{deg[8]='♭6';alts.push('♭6')}}
  if(h(9)&&deg[9]==null){if(sev){deg[9]='13';nat.push(13)}else{deg[9]='6';nat.push(6)}}
  if(h(10)&&deg[10]==null){deg[10]='♭7';alts.push('♭7')}
  const hi=sev&&sev!=='dim'?Math.max(7,...nat.filter(x=>x>=9)):7;
  let s=N[root];
  if(dim)s+=sev==='dim'?'°7':sev==='dom'?'ø7':sev==='maj'?'°(maj7)':'°';
  else{
    if(third==='m')s+='m';
    if(sev==='maj')s+=third==='m'?'(maj'+hi+')':'maj'+hi;
    else if(sev==='dom')s+=hi;
    else{
      if(aug)s+='+';
      const six=nat.includes(6),nine=nat.includes(9);
      s+=six&&nine?'6/9':six?'6':nine?'add9':'';
      if(nat.includes(11))s+='add11';
    }
  }
  const altList=(aug&&sev?['♯5']:[]).concat(alts);
  if(third==='sus4')s+='sus4';if(third==='sus2')s+='sus2';
  if(altList.length)s+=sev?altList.join(''):'('+altList.join(',')+')';
  if(third==='no3')s+='(no3)';
  if(bassPc!==root)s+='/'+N[bassPc];
  const rootPresent=h(0);
  let sc=(rootPresent?3:-1.5)+(root===bassPc?2:0)+(third==='M'||third==='m'?2:third==='no3'?-2.5:-0.5)+(P5?1:dim||aug?0.3:0)+(sev?0.8:0)-0.4*nat.length-0.9*altList.length;
  return {root,name:s,deg,score:sc,rootPresent,third,sev,P5,alts:altList.length,rel};
}

function analyze(notes,o){
  const {N,lim,A4,timbre}=o, n=notes.length;
  const bass=notes[0],bassPc=bass%12;
  const pcs=[...new Set(notes.map(m=>m%12))];
  const et=m=>A4*2**((m-69)/12);
  const rats=notes.map(m=>ratioFor(m-bass,lim));
  const L=rats.reduce((a,r)=>lcm(a,r[1]),1);
  let ints=rats.map(([p,q])=>p*L/q);const g=ints.reduce((a,b)=>gcd(a,b));ints=ints.map(x=>x/g);
  const P=ints[0],fb=et(bass),f0=fb/P,period=P/fb;
  const justF=rats.map(([p,q])=>fb*p/q),etF=notes.map(et);
  const cents=notes.map((m,i)=>1200*Math.log2(etF[i]/justF[i]));
  const all=[...Array(12).keys()].map(r=>reading(r,pcs,bassPc,N)).sort((a,b)=>b.score-a.score);
  const ex=all.map(r=>Math.exp(r.score*0.8)),Z=ex.reduce((a,b)=>a+b);all.forEach((r,i)=>r.p=ex[i]/Z);
  const best=all[0];
  const fr=o.tuning==='just'?justF:etF;
  const ref=pairRough(et(60),et(61),timbre);
  const pairs=[];
  for(let i=0;i<n;i++)for(let j=i+1;j<n;j++)pairs.push({i,j,d:notes[j]-notes[i],r:clamp(pairRough(fr[i],fr[j],timbre)/ref)});
  const sumR=pairs.reduce((a,p)=>a+p.r,0);
  const tension=clamp(1-Math.exp(-sumR*0.9));
  let harsh=0;
  pairs.forEach(p=>{const ic=p.d%12;if(ic===1||ic===11)harsh+=p.d===1?1:p.d===13?0.75:p.d===11?0.55:0.35;else if(ic===6)harsh+=0.25;else if(p.d===2)harsh+=0.3});
  const fifthFrame=pairs.some(p=>p.d%12===7);
  harsh=clamp(harsh/1.6*(fifthFrame?1.2:1));
  const rel=best.rel,cross=rel.has(3)&&rel.has(4),b9dom=rel.has(1)&&rel.has(4);
  const ambiguity=pcs.length<2?0:clamp(all[1].p/all[0].p*1.1);
  const Pn=clamp(Math.log2(P)/7);
  const complexity=clamp(0.4*Pn+0.25*clamp(best.alts/3)+0.15*clamp((pcs.length-3)/4)+0.1*ambiguity+(!fifthFrame&&tension>0.3?0.15:0));
  let br=0,c=0;rel.forEach(ic=>{if(ic){br+=BRIGHT[ic];c++}});br=c?br/c:0;
  const reg=(notes.reduce((a,b)=>a+b)/n-60)/24;
  const brightness=clamp(0.5+br/2.4+reg*0.25);
  const stability=clamp(0.3+(fifthFrame?0.25:0)+(best.root===bassPc?0.2:0)+(best.rootPresent?0.1:0)-0.35*tension-0.15*ambiguity+(P<=6?0.15:0));
  const aggression=clamp(0.45*(cross?1:0)+0.3*harsh+0.25*tension+(b9dom?0.25:0));
  const valence=clamp(brightness*0.6+stability*0.4-0.25*harsh);
  const arousal=clamp(0.5*tension+0.3*aggression+0.2*complexity);
  const R=best.root,nm=i=>N[(R+i)%12],tags=[];
  if(cross||b9dom){
    tags.push({w:'aggressive',k:1,why:cross?`Major 3rd (${nm(4)}) against ♯9 (${nm(3)}): one scale degree in two colours at once, a cross-relation a semitone apart.`:`♭9 (${nm(1)}) over a major-3rd dominant: the root is shadowed a semitone above.`});
    if(rel.has(7))tags.push({w:'harsh',k:0.95,why:`The pure 5th (${nm(0)}–${nm(7)}) keeps the frame stable, so the clash is exposed rather than blended.`});
    else if(rel.has(8))tags.push({w:'complex',k:0.95,why:`The ♯5 (${nm(8)}) replaces the pure 5th. With no stable frame the clash reads as layered and unresolved.`});
  }
  const close=pairs.find(p=>p.d===1);
  if(close)tags.push({w:'grinding',k:0.85,why:`${N[notes[close.i]%12]}–${N[notes[close.j]%12]} sit a semitone apart in the same register: near-maximal roughness.`});
  if(ambiguity>0.55)tags.push({w:'ambiguous',k:0.8,why:`Reads almost equally as ${all[0].name} and ${all[1].name}; the ear can hear either root.`});
  if(best.third==='m'&&rel.has(8)&&!best.sev)tags.push({w:'dark',k:0.7,why:`Minor 3rd plus ♭6 (${nm(8)}): both are lowered neighbours, the darkest pairing over a minor frame.`});
  if(best.sev==='maj'&&best.third==='M'&&!cross)tags.push({w:'lush',k:0.7,why:`Major 7th (${nm(11)}) sits a semitone under the octave; the friction stays soft because the triad beneath is consonant.`});
  if(rel.has(4)&&rel.has(10))tags.push({w:'driving',k:0.65,why:`Tritone between 3 (${nm(4)}) and ♭7 (${nm(10)}) pulls inward toward resolution.`});
  if(best.deg[6]==='♯11')tags.push({w:'luminous',k:0.6,why:`♯11 (${nm(6)}) raises the 4th, removing its friction with the 3rd.`});
  if(best.third==='sus4'||best.third==='sus2')tags.push({w:'suspended',k:0.6,why:'No 3rd: the chord withholds major or minor.'});
  const adj=notes.slice(1).map((m,i)=>m-notes[i]);
  if(n>=3&&adj.every(d=>d===5||d===6))tags.push({w:'open',k:0.6,why:'Stacked 4ths: no triadic centre, evenly spaced and modern.'});
  if(n>=3&&adj.every(d=>d===3))tags.push({w:'unstable',k:0.7,why:'Stacked minor 3rds divide the octave symmetrically; every note could be the root.'});
  if(P<=6&&n>1)tags.push({w:'fused',k:0.5,why:`Low periodicity: the notes line up as harmonics ${ints.join(':')}, heard almost as one tone-colour.`});
  else if(P>=40)tags.push({w:'cloudy',k:0.5,why:`Long common period (${P} bass cycles): the summed wave takes a long time to repeat.`});
  if(!tags.length&&best.third==='M'&&best.P5)tags.push({w:'bright',k:0.4,why:'Major 3rd over a pure 5th: the reference consonance of the major triad.'});
  if(!tags.length&&best.third==='m'&&best.P5)tags.push({w:'somber',k:0.4,why:'Minor 3rd over a pure 5th: stable, but shaded.'});
  tags.sort((a,b)=>b.k-a.k);
  return {notes,n,pcs,bassPc,rats,ints,P,f0,period,justF,etF,cents,all,best,pairs,tags,
    axes:{tension,harsh,aggression,complexity,brightness,stability,ambiguity},valence,arousal};
}
