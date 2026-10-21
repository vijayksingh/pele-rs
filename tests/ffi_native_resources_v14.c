#include "pele.h"
#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
_Static_assert(sizeof(PeleNativeResourcesV14)==64,"v14 packet size");
_Static_assert(offsetof(PeleNativeResourcesV14,image_loader)==16,"v14 loader offset");
_Static_assert(offsetof(PeleNativeResourcesV14,reserved)==48,"v14 reserved offset");
static const char *source="flowchart LR\nA@{img: \"https://example.test/a.png\",label: \"Image\",w:80,h:60} --> B";
static const unsigned char png[]={137,80,78,71,13,10,26,10,0,0,0,13,73,72,68,82,0,0,0,1,0,0,0,1,8,6,0,0,0,0,0,0,0};
struct State{unsigned calls;int mode;PelePublicDocument *active;};
static char *body(PeleResult *r){assert(r);size_t n=pele_result_len(r);assert(n<=64*1024*1024);char *s=malloc(n+1);assert(s);memcpy(s,pele_result_data(r),n);s[n]=0;return s;}
static void check(PeleResult *r,unsigned status,const char *part){assert(r&&pele_result_status(r)==status);char *s=body(r);if(!strstr(s,part))fprintf(stderr,"missing %s in %.1024s\n",part,s);assert(strstr(s,part));free(s);pele_result_free(r);}
static uint32_t image(void *raw,const uint8_t *uri,size_t length,PeleBorrowedBytes *output){
 struct State *s=raw;s->calls++;assert(length==strlen("https://example.test/a.png"));assert(!memcmp(uri,"https://example.test/a.png",length));
 if(s->active){PeleResult *snapshot=pele_public_snapshot_binary_v5(s->active);assert(snapshot&&pele_result_status(snapshot)==0);pele_result_free(snapshot);}
 if(s->mode==1)return 1;if(s->mode==2){output->data=NULL;output->len=0;return 0;}
 output->data=png;output->len=sizeof(png);return 0;
}
static PeleResult *render(const char *s,const char *options,PeleNativeResourcesV14 *p){return pele_render_native_v14((const uint8_t*)s,strlen(s),(const uint8_t*)options,strlen(options),NULL,0,0,NULL,NULL,p);}
static uint64_t le64(const uint8_t *p){uint64_t n=0;for(unsigned i=0;i<8;i++)n|=(uint64_t)p[i]<<(i*8);return n;}
static int contains(const uint8_t *p,size_t n,const char *part){size_t m=strlen(part);for(size_t i=0;m<=n&&i<=n-m;i++)if(!memcmp(p+i,part,m))return 1;return 0;}
static void outcome(PeleResult *r,uint64_t revision,const char *part){assert(r&&pele_result_status(r)==0);const uint8_t *p=pele_result_data(r);size_t n=pele_result_len(r);assert(n>=24&&!memcmp(p,"PELGBIN1",8));assert(le64(p+12)==revision);assert(contains(p,n,part));pele_result_free(r);}
int main(void){
 struct State s={0};PeleNativeResourcesV14 p={.version=14,.struct_size=sizeof(p),.context=&s,.image_loader=image,.max_image_bytes=8*1024*1024,.max_image_pixels=16777216,.max_total_bytes=16*1024*1024};
 check(render(source,"{}",&p),0,"data:image/png;base64,");assert(s.calls==1);
 p.max_total_bytes=sizeof(png);unsigned calls=s.calls;
 check(render("flowchart LR\nA@{img: \"https://example.test/a.png\",w:80,h:60} --> B@{img: \"https://example.test/a.png\",w:80,h:60}","{}",&p),2,"encoded byte budget");assert(s.calls==calls+1);
 p.max_total_bytes=16*1024*1024;s.calls=1;

 check(render(source,"{\"images\":false}",&p),0,"Image");assert(s.calls==1);
 check(render("flowchart LR\nA@{img: \"javascript:bad\",label: \"Image\"}","{}",&p),0,"Image");assert(s.calls==1);
 check(render(source,"{}",NULL),1,"approved host image loader");
 p.max_image_bytes=1;check(render(source,"{}",&p),2,"encoded byte budget");p.max_image_bytes=8*1024*1024;
 p.max_image_pixels=0;check(render(source,"{}",&p),2,"decoded pixel budget");p.max_image_pixels=16777216;
 p.version=13;check(render(source,"{}",&p),1,"version or size");p.version=14;
 p.reserved[0]=1;check(render(source,"{}",&p),1,"reserved");p.reserved[0]=0;
 s.mode=1;check(render(source,"{}",&p),1,"image loader failed");s.mode=2;check(render(source,"{}",&p),1,"did not approve");s.mode=0;
 PelePublicDocument *document=NULL;check(pele_public_parse_v4((const uint8_t*)source,strlen(source),NULL,0,&document),0,"flowchart");assert(document);s.active=document;
 outcome(pele_public_render_native_v14(document,0,NULL,0,NULL,0,0,NULL,NULL,&p),1,"data:image/png;base64,");
 s.mode=1;outcome(pele_public_render_native_v14(document,1,NULL,0,NULL,0,0,NULL,NULL,&p),2,"image loader failed");
 s.mode=0;outcome(pele_public_render_native_v14(document,2,NULL,0,NULL,0,0,NULL,NULL,&p),3,"data:image/png;base64,");
 check(pele_public_render_native_v14(document,2,NULL,0,NULL,0,0,NULL,NULL,&p),1,"Stale");pele_public_document_free_v4(document);
 puts("native resources v14 integration passed");return 0;
}
