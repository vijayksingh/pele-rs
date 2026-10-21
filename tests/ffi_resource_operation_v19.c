#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static void ok(PeleResult *r){assert(r&&pele_result_status(r)==0);pele_result_free(r);}
static void rejects(PeleResult *r){assert(r&&pele_result_status(r)!=0);pele_result_free(r);}
static char *metadata(PeleResult *r,uint64_t *revision){
 assert(r&&pele_result_status(r)==0);const uint8_t *b=pele_result_data(r);const size_t n=pele_result_len(r);assert(n>=24&&!memcmp(b,"PELGBIN1",8));*revision=0;for(unsigned i=0;i<8;i++)*revision|=(uint64_t)b[12+i]<<(8*i);
 const size_t m=(size_t)b[20]|(size_t)b[21]<<8|(size_t)b[22]<<16|(size_t)b[23]<<24;assert(m<=n-24);char *text=malloc(m+1);assert(text);memcpy(text,b+n-m,m);text[m]=0;pele_result_free(r);return text;
}
static unsigned receiver_id;
static unsigned source_argument(PelePublicDocument *document){
 PeleResult *r=pele_public_snapshot_v4(document);assert(r&&pele_result_status(r)==0);const size_t n=pele_result_len(r);char *text=malloc(n+1);assert(text);memcpy(text,pele_result_data(r),n);text[n]=0;
 const char *root=strstr(text,"\"root\":{\"kind\":\"ref\",\"value\":");assert(root);root+=strlen("\"root\":{\"kind\":\"ref\",\"value\":");char *end=NULL;const unsigned long receiver=strtoul(root,&end,10);assert(end!=root&&*end=='}'&&receiver<=UINT32_MAX);receiver_id=(unsigned)receiver;
 const char *p=strstr(text,"\"strings\":[");assert(p);p+=strlen("\"strings\":[");unsigned id=0;const char *wanted="[97,102,116,101,114,32,109,105,115,115,105,110,103]";
 while(*p=='['&&strncmp(p,wanted,strlen(wanted))){p=strchr(p,']');assert(p);p++;if(*p==',')p++;id++;}assert(!strncmp(p,wanted,strlen(wanted)));free(text);pele_result_free(r);return id;
}
struct Clock{PelePublicDocument *document;unsigned calls;};
static uint32_t now(void *opaque,double *out){struct Clock *clock=opaque;clock->calls++;ok(pele_public_snapshot_v4(clock->document));ok(pele_public_schema_v4(clock->document));*out=1704110400000.;return 0;}
static uint32_t offset(void *opaque,double instant,int32_t *out){struct Clock *clock=opaque;(void)instant;clock->calls++;ok(pele_public_snapshot_v4(clock->document));ok(pele_public_schema_v4(clock->document));*out=0;return 0;}
static PeleInvocationJobV17 *prepare(PelePublicDocument *document,uint64_t revision,unsigned argument){
 char call[256];const int n=snprintf(call,sizeof(call),"{\"method\":28,\"receiver\":{\"kind\":\"ref\",\"value\":%u},\"arguments\":[{\"kind\":\"string\",\"value\":%u}]}",receiver_id,argument);assert(n>0&&(size_t)n<sizeof(call));PeleInvocationJobV17 *job=NULL;ok(pele_public_invocation_prepare_v17(document,revision,(const uint8_t *)call,(size_t)n,&job));assert(job);return job;
}
static void until_resource(PelePublicDocument *document,PeleInvocationJobV17 *job,uint64_t *revision){
 PeleHostValueV17 undefined={0};undefined.version=17;undefined.struct_size=sizeof(undefined);
 for(unsigned i=0;i<32;i++){
  char *step=metadata(pele_public_invocation_resume_v17(document,job,i?1:0,&undefined,revision),revision);
  if(strstr(step,"\"resource\":true")){free(step);return;}
  assert(strstr(step,"\"host\":"));
  if(strstr(step,"\"kind\":\"get\"")){undefined.tag=0;undefined.payload=0;}
  else {assert(strstr(step,"\"kind\":\"truthy\""));undefined.tag=2;undefined.payload=0;}
  free(step);
 }assert(!"no resource step");
}
int main(void){
 const char *source="gantt\ndateFormat YYYY-MM-DD\nsection Test\nafter missing:a,2024-01-01,1d\n";PelePublicDocument *document=NULL;ok(pele_public_parse_v4((const uint8_t *)source,strlen(source),NULL,0,&document));const unsigned argument=source_argument(document);uint64_t revision=0;
 PeleInvocationJobV17 *job=prepare(document,revision,argument);until_resource(document,job,&revision);PeleResourceOperationV19 *resource=NULL;ok(pele_public_invocation_take_resource_v19(job,&resource));assert(resource);PeleResourceOperationV19 *duplicate=NULL;rejects(pele_public_invocation_take_resource_v19(job,&duplicate));assert(!duplicate);
 PeleInvocationJobV17 *other=prepare(document,revision,argument);rejects(pele_public_invocation_accept_resource_v19(other,resource));pele_public_invocation_free_v17(other);other=NULL;rejects(pele_public_invocation_accept_resource_v19(job,resource));
 struct Clock clock={document,0};PeleHostResourcesV3 host={0};host.base.context=&clock;host.base.now=now;host.utc_offset=offset;ok(pele_resource_operation_run_v19(resource,&host));assert(clock.calls>0);rejects(pele_resource_operation_run_v19(resource,&host));ok(pele_public_invocation_accept_resource_v19(job,resource));rejects(pele_public_invocation_accept_resource_v19(job,resource));
 char *step=metadata(pele_public_invocation_resume_v17(document,job,4,NULL,&revision),&revision);assert(strstr(step,"\"complete\":true"));free(step);char *finish=metadata(pele_public_invocation_finish_v17(document,job,&revision),&revision);assert(strstr(finish,"\"Ok\":{\"kind\":\"ref\""));free(finish);pele_resource_operation_free_v19(resource);pele_public_invocation_free_v17(other);pele_public_invocation_free_v17(job);
 job=prepare(document,revision,argument);until_resource(document,job,&revision);resource=NULL;ok(pele_public_invocation_take_resource_v19(job,&resource));pele_resource_operation_free_v19(resource);pele_public_invocation_free_v17(job);ok(pele_public_snapshot_v4(document));pele_public_document_free_v4(document);puts("native resource operation lifecycle passed");return 0;
}
