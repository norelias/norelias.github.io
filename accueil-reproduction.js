(()=>{
 'use strict';
 const burger=document.querySelector('.burger'),nav=document.querySelector('.mainnav');
 if(burger&&nav){burger.addEventListener('click',()=>{const expanded=burger.getAttribute('aria-expanded')==='true';burger.setAttribute('aria-expanded',String(!expanded));nav.classList.toggle('open',!expanded)});nav.addEventListener('click',e=>{if(e.target.closest('a')){nav.classList.remove('open');burger.setAttribute('aria-expanded','false')}})}
 const photos=(window.NORELIAS_GALLERY||[]).slice(0,12),stage=document.querySelector('#gallery-tiles'),dots=document.querySelector('.gallery-dots');
 const modal=document.querySelector('.lightbox'),modalImg=document.querySelector('.lightbox-image');let cursor=2,modalIndex=0;
 const sizesDesktop=[{x:-464,y:21,w:180,h:146,rot:-4,z:1},{x:-294,y:11,w:163,h:157,rot:-2,z:2},{x:-143,y:0,w:286,h:186,rot:0,z:4},{x:138,y:12,w:161,h:158,rot:3,z:3},{x:286,y:21,w:182,h:151,rot:4,z:1}];
 // Cards are dynamically assembled from separate <img> elements.
 // Desktop shows five overlapping photos, mobile shows a center and two partial glimpses.
 function render(){if(!stage)return;stage.textContent='';let mobile=window.innerWidth<=760;
  const specs=mobile?[{x:-179,y:7,w:76,h:58,rot:-4,z:1},{x:-121,y:3,w:76,h:63,rot:-2,z:2},{x:-52,y:0,w:103,h:67,rot:0,z:4},{x:45,y:3,w:76,h:63,rot:3,z:3},{x:103,y:7,w:76,h:58,rot:4,z:1}]:sizesDesktop;
  specs.forEach((spec,j)=>{const offset=j-2,index=(cursor+offset+photos.length)%photos.length;const p=photos[index];const b=document.createElement('button');b.type='button';b.className='photo-card'+(j===2?' center':'');b.setAttribute('aria-label',(j===2?'Agrandir ':'Afficher ')+p.alt);b.style.cssText=`width:${spec.w}px;height:${spec.h}px;left:calc(50% + ${spec.x}px);top:${spec.y}px;z-index:${spec.z};transform:rotate(${spec.rot}deg);`;const img=document.createElement('img');img.src=p.src;img.alt=p.alt;img.decoding='async';b.append(img);b.addEventListener('click',()=>{if(offset){cursor=index;render()}else openModal(index)});stage.append(b)});
  if(dots){dots.textContent='';for(let j=0;j<photos.length;j++){const b=document.createElement('button');b.type='button';if(j===cursor)b.className='active';b.setAttribute('aria-label','Aller à la photo '+(j+1));b.addEventListener('click',()=>{cursor=j;render()});dots.append(b)}}
 }
 const step=dir=>{cursor=(cursor+dir+photos.length)%photos.length;render()};
 let galPause=false,galTimer=null;const galEl=document.querySelector('.gallery');const galStart=()=>{clearInterval(galTimer);galTimer=setInterval(()=>{if(galPause||document.hidden||(modal&&modal.getAttribute('aria-hidden')==='false'))return;step(1)},3500)};if(galEl&&!window.matchMedia('(prefers-reduced-motion: reduce)').matches){galEl.addEventListener('mouseenter',()=>galPause=true);galEl.addEventListener('mouseleave',()=>galPause=false);galEl.addEventListener('touchstart',()=>{galPause=true},{passive:true});galEl.addEventListener('touchend',()=>{setTimeout(()=>{galPause=false},4000)},{passive:true});galStart()}
 document.querySelector('.gallery-arrow.left')?.addEventListener('click',()=>step(-1));document.querySelector('.gallery-arrow.right')?.addEventListener('click',()=>step(1));
 const gallery=document.querySelector('.gallery');let tx=null;gallery?.addEventListener('touchstart',e=>{tx=e.touches[0].clientX},{passive:true});gallery?.addEventListener('touchend',e=>{if(tx!==null){const d=e.changedTouches[0].clientX-tx;if(Math.abs(d)>42)step(d<0?1:-1);tx=null}},{passive:true});gallery?.addEventListener('keydown',e=>{if(e.key==='ArrowRight'){step(1);e.preventDefault()}if(e.key==='ArrowLeft'){step(-1);e.preventDefault()}});
 function openModal(i){modalIndex=i;if(!modal||!modalImg)return;modalImg.src=photos[i].src;modalImg.alt=photos[i].alt;modal.setAttribute('aria-hidden','false');document.body.style.overflow='hidden';modal.querySelector('.lightbox-close')?.focus()}
 function closeModal(){if(modal){modal.setAttribute('aria-hidden','true');document.body.style.overflow=''}}
 function modalStep(d){modalIndex=(modalIndex+d+photos.length)%photos.length;modalImg.src=photos[modalIndex].src;modalImg.alt=photos[modalIndex].alt}
 modal?.querySelector('.lightbox-close')?.addEventListener('click',closeModal);modal?.querySelector('.lightbox-prev')?.addEventListener('click',()=>modalStep(-1));modal?.querySelector('.lightbox-next')?.addEventListener('click',()=>modalStep(1));modal?.addEventListener('click',e=>{if(e.target===modal)closeModal()});
 document.addEventListener('keydown',e=>{if(modal?.getAttribute('aria-hidden')==='false'){if(e.key==='Escape')closeModal();if(e.key==='ArrowLeft')modalStep(-1);if(e.key==='ArrowRight')modalStep(1)}});
 window.addEventListener('resize',()=>{render()},{passive:true});render();
})();
(()=>{const l=document.querySelector('.review-list');if(!l)return;let pause=false;
 l.addEventListener('mouseenter',()=>pause=true);l.addEventListener('mouseleave',()=>pause=false);l.addEventListener('touchstart',()=>pause=true,{passive:true});
 setInterval(()=>{if(pause||document.hidden)return;const c=l.querySelector('.review-card');if(!c)return;const w=c.getBoundingClientRect().width/(parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--z'))||1)+13;
  const max=l.scrollWidth-l.clientWidth;l.scrollTo({left:l.scrollLeft>=max-4?0:l.scrollLeft+c.offsetWidth+13,behavior:'smooth'})},4500)})();
