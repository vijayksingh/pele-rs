#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void okay(PeleResult *r) { assert(r && pele_result_status(r)==0); pele_result_free(r); }
static char *json(PeleResult *r) {
    assert(r && pele_result_status(r)==0); size_t n=pele_result_len(r);
    char *s=malloc(n+1); assert(s); memcpy(s,pele_result_data(r),n); s[n]=0; pele_result_free(r); return s;
}
static uint64_t revision(PelePublicDocument *d) {
    char *s=json(pele_public_revision_v15(d)); char *p=strstr(s,"\"revision\":\""); assert(p);
    uint64_t v=strtoull(p+strlen("\"revision\":\""),NULL,10); free(s); return v;
}
static char *metadata(PeleResult *r) {
    assert(r && pele_result_status(r)==0); const uint8_t *p=pele_result_data(r); size_t n=pele_result_len(r);
    assert(n>=24 && !memcmp(p,"PELGBIN1",8)); size_t len=(size_t)p[20]|(size_t)p[21]<<8|(size_t)p[22]<<16|(size_t)p[23]<<24;
    assert(len<=n-24); char *s=malloc(len+1); assert(s); memcpy(s,p+n-len,len);s[len]=0;pele_result_free(r);return s;
}
static PeleHostValueV17 value(uint32_t tag,uint64_t bits) {
    PeleHostValueV17 v={0};v.version=17;v.struct_size=sizeof(v);v.tag=tag;v.payload=bits;return v;
}
/* Balanced arrays/objects from public JSON, never private arena addresses. */
static char *end_value(char *p) {
    char close=*p=='['?']':'}'; unsigned depth=1; int quoted=0,escaped=0;
    for(++p;*p;++p) {
        if(quoted){if(escaped)escaped=0;else if(*p=='\\')escaped=1;else if(*p=='"')quoted=0;continue;}
        if(*p=='"'){quoted=1;continue;}
        if(*p=='['||*p=='{')++depth;
        else if(*p==']'||*p=='}'){if(--depth==0){assert(*p==close);return p;}}
    } assert(!"incomplete JSON"); return NULL;
}
static uint32_t identity(void) {
    char *s=json(pele_host_identity_v17());
    const char *prefix="{\"version\":2,\"ok\":true,\"result\":";
    assert(!strncmp(s,prefix,strlen(prefix)));
    char *end=NULL;unsigned long id=strtoul(s+strlen(prefix),&end,10);
    assert(end && !strcmp(end,"}") && id>0 && id<=UINT32_MAX);
    free(s);return (uint32_t)id;
}
static PelePublicDocument *construct(uint32_t options) {
    PeleRegistry *registry=NULL;okay(pele_registry_create_v8(1,&registry));
    PelePublicConstructorV16 *factory=NULL;
    okay(pele_registry_capture_constructor_v16(registry,(const uint8_t *)"agentflow",9,1,&factory));assert(factory);
    PeleHostValueV17 argument=value(5,options);PeleConstructionJobV18 *job=NULL;
    okay(pele_public_constructor_begin_v18(factory,&argument,NULL,&job));assert(job);
    uint64_t rev=0;char *step=metadata(pele_public_construction_resume_v18(job,0,NULL,&rev));
    assert(strstr(step,"lineOffset"));free(step);argument=value(0,0);
    step=metadata(pele_public_construction_resume_v18(job,1,&argument,&rev));assert(strstr(step,"\"complete\":true"));free(step);
    PelePublicDocument *d=NULL;okay(pele_public_construction_finish_v18(job,&d,&rev));assert(d);
    pele_public_construction_free_v18(job);pele_public_constructor_free_v16(factory);pele_registry_free_v8(registry);
    char *snapshot=json(pele_public_snapshot_v4(d));char *graph=strstr(snapshot,"\"graph\":");assert(graph);graph+=8;
    char *last=end_value(graph);last[1]=0;
    char *root=strstr(graph,"\"root\":{\"kind\":\"ref\",\"value\":");assert(root);
    unsigned root_id=(unsigned)strtoul(root+strlen("\"root\":{\"kind\":\"ref\",\"value\":"),NULL,10);
    char *strings=strstr(graph,"\"strings\":[");assert(strings);strings+=10;unsigned count=0;
    for(char *p=strings+1;*p!=']';){assert(*p=='[');p=end_value(p)+1;++count;if(*p==',')++p;}
    char *end=end_value(strings);const char *addition=",[65],[99,105,114,99,108,101]";
    size_t prefix=(size_t)(end-graph), extra=strlen(addition), suffix=strlen(end);
    char *edited=malloc(prefix+extra+suffix+1);assert(edited);
    memcpy(edited,graph,prefix);memcpy(edited+prefix,addition,extra);memcpy(edited+prefix+extra,end,suffix+1);
    okay(pele_public_apply_v4(d,revision(d),(const uint8_t *)edited,strlen(edited)));free(edited);free(snapshot);
    char call[512];int length=snprintf(call,sizeof(call),"{\"method\":3,\"receiver\":{\"kind\":\"ref\",\"value\":%u},\"arguments\":[{\"kind\":\"string\",\"value\":%u},{\"kind\":\"undefined\"},{\"kind\":\"string\",\"value\":%u}]}",root_id,count,count+1);
    assert(length>0 && (size_t)length<sizeof(call));
    char *result=json(pele_public_invoke_v4(d,revision(d),(const uint8_t *)call,(size_t)length,NULL));assert(strstr(result,"\"result\":{\"Ok\""));free(result);
    return d;
}
struct State { PelePublicDocument *document; unsigned gets,calls,retains,releases,replies,native_calls; uint32_t options,logger,exception; int mode,entered; };
static void retain(void *p){((struct State *)p)->retains++;}
static void release(void *p){((struct State *)p)->releases++;}
static void reply_release(void *p){((struct State *)p)->replies++;}
static void draw(struct State *s,int expected_throw);
static void invoke(void *p,const PeleHostRequestV20 *request,PeleHostReplyV20 *reply) {
    struct State *s=p;okay(pele_public_schema_v4(s->document));okay(pele_public_snapshot_v4(s->document));
    reply->owner=s;reply->release=reply_release;
    reply->status=1;reply->value=value(0,0);
    if(request->operation==0){
        assert(request->count==1 && request->values[0].tag==5 && request->values[0].payload==s->options);
        assert(request->key_count==3 && request->key[0]=='l' && request->key[1]=='o' && request->key[2]=='g');
        ++s->gets;
        if(s->mode==2){reply->status=2;reply->exception=s->exception;return;}
        reply->value=value(5,s->logger);return;
    }
    assert(request->operation==1 && request->count==4);
    assert(request->values[0].tag==5 && request->values[0].payload==s->logger);
    assert(request->values[1].tag==5 && request->values[1].payload==s->options);
    assert(request->values[2].tag==4 && request->values[3].tag==4);
    ++s->calls;
    if(s->mode==4 || s->mode==5){reply->status=0;return;}
    if(s->mode==6){reply->status=2;reply->exception=s->exception;return;}
    if(!s->entered){s->entered=1;if(s->mode==1)draw(s,0);}
    if(s->mode==3){reply->status=2;reply->exception=s->exception;return;}
    assert(request->result==2); /* The source ignores diagnostic return values. */
    reply->value=value(99,0); /* Discard must not decode an ignored return. */
}
static void draw(struct State *s,int expected_throw) {
    PeleDetachedRenderJobV15 *job=NULL;uint64_t before=revision(s->document),ack=before;
    okay(pele_public_prepare_detached_render_v15(s->document,before,NULL,0,NULL,3,0,&job));assert(job);
    okay(pele_detached_render_run_v15(job,NULL));
    char *done=metadata(pele_public_finish_detached_render_v15(s->document,job,&ack));
    assert(strstr(done,expected_throw==2?"\"Err\"":expected_throw?"hostException":"\"Ok\""));free(done);
    assert(ack>before && ack==revision(s->document));pele_detached_render_job_free_v15(job);
}
static void native_call(void *p,PeleActiveModelV12 *active,const uint8_t *bytes,size_t length,PeleHostFunctionReplyV12 *reply) {
    (void)active;(void)bytes;(void)length;struct State *s=p;++s->native_calls;
    okay(pele_public_snapshot_v4(s->document));
    static const uint8_t returned[]="{\"kind\":\"undefined\"}";
    reply->status=0;reply->payload.data=returned;reply->payload.len=sizeof(returned)-1;
}
int main(void) {
    for(int mode=0;mode<7;++mode){
        struct State state={0};state.options=identity();state.logger=identity();state.exception=identity();state.document=construct(state.options);state.mode=mode;
        PelePublicDocument *foreign=NULL;
        if(mode==4 || mode==5){
            if(mode==5)foreign=construct(identity());
            PeleHostFunctionV12 function={0};function.version=12;function.struct_size=sizeof(function);function.context=&state;
            function.retain=retain;function.release=release;function.invoke=native_call;
            okay(pele_public_bind_host_function_v12(foreign?foreign:state.document,&function,&state.logger));
        }
        PeleHostAccessV20 table={0};table.version=20;table.struct_size=sizeof(table);table.context=&state;table.retain=retain;table.release=release;table.invoke=invoke;
        okay(pele_public_set_host_access_v20(state.document,&table));assert(state.retains==(mode==4 || mode==5?2u:1u));
        draw(&state,mode==5?2:mode==2 || mode==3 || mode==6);assert(state.gets>0);if(mode!=2)assert(state.calls>0);
        if(mode==2 || mode==3 || mode==5 || mode==6){state.mode=0;state.logger=identity();draw(&state,0);}
        assert(state.replies==state.gets+state.calls);
        okay(pele_public_set_host_access_v20(state.document,NULL));assert(state.releases==1);
        assert(state.native_calls==(mode==4?1u:0u));
        pele_public_document_free_v4(state.document);
        if(foreign)pele_public_document_free_v4(foreign);
        assert(state.releases==state.retains);
    }
    puts("C20 opaque Get/Call, owned constructor, nested render, original errors and recovery PASS");return 0;
}
