#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void okay(PeleResult *r) { assert(r && pele_result_status(r) == 0); pele_result_free(r); }
static void failure(PeleResult *r, const char *message) {
    assert(r && pele_result_status(r) != 0);
    char *text = malloc(pele_result_len(r) + 1); assert(text);
    memcpy(text, pele_result_data(r), pele_result_len(r)); text[pele_result_len(r)] = 0;
    assert(strstr(text, message)); free(text); pele_result_free(r);
}
static char *metadata(PeleResult *r, uint64_t revision) {
    assert(r && pele_result_status(r) == 0);
    const uint8_t *p = pele_result_data(r); size_t n = pele_result_len(r);
    assert(n >= 24 && !memcmp(p, "PELGBIN1", 8));
    uint64_t actual = 0; for (unsigned i = 0; i < 8; ++i) actual |= (uint64_t)p[12+i] << (8*i);
    assert(actual == revision);
    size_t length = (size_t)p[20] | (size_t)p[21]<<8 | (size_t)p[22]<<16 | (size_t)p[23]<<24;
    assert(length <= n-24); char *text = malloc(length+1); assert(text);
    memcpy(text, p+n-length, length); text[length]=0; pele_result_free(r); return text;
}
static unsigned root_id(PelePublicDocument *d) {
    PeleResult *r = pele_public_snapshot_v4(d); assert(r && pele_result_status(r)==0);
    char *s = malloc(pele_result_len(r)+1); assert(s); memcpy(s,pele_result_data(r),pele_result_len(r));s[pele_result_len(r)]=0;
    const char *key="\"root\":{\"kind\":\"ref\",\"value\":"; char *p=strstr(s,key); assert(p);
    unsigned id=(unsigned)strtoul(p+strlen(key),NULL,10);free(s);pele_result_free(r);return id;
}
static PeleHostValueV17 value(uint32_t tag, uint64_t payload) {
    PeleHostValueV17 v={0};v.version=17;v.struct_size=sizeof(v);v.tag=tag;v.payload=payload;return v;
}
static PeleInvocationJobV17 *prepare(PelePublicDocument *d,uint64_t revision) {
    char call[256]; int n=snprintf(call,sizeof(call),"{\"method\":24,\"receiver\":{\"kind\":\"ref\",\"value\":%u},\"arguments\":[{\"kind\":\"hostReference\",\"value\":11}]}",root_id(d));
    assert(n>0 && (size_t)n<sizeof(call));PeleInvocationJobV17 *job=NULL;
    okay(pele_public_invocation_prepare_v17(d,revision,(const uint8_t *)call,(size_t)n,&job));assert(job);return job;
}
static void drive(PelePublicDocument *d,uint64_t *revision,int nested,int throw_producer) {
    PeleInvocationJobV17 *job=prepare(d,*revision);
    PeleHostValueV17 reply=value(0,0);uint32_t state=0;unsigned gets=0,calls=0;int entered=0;
    for (;;) {
        uint64_t before=*revision;
        PeleResult *response=pele_public_invocation_resume_v17(d,job,state,&reply,revision);
        assert(*revision>before);char *step=metadata(response,*revision);
        if (strstr(step,"\"complete\":true")) {free(step);break;}
        okay(pele_public_schema_v4(d)); /* No document borrow spans a host phase. */
        reply=value(0,0);state=1;
        if(strstr(step,"\"kind\":\"get\"")) {
            ++gets;
            if(strstr(step,"\"key\":\"options\"")) reply=value(5,10);
            else {assert(strstr(step,"\"key\":\"warn\""));reply=nested?value(2,0):value(5,20);}
        } else if(strstr(step,"\"kind\":\"truthy\"")) {reply=value(2,nested?0:1);}
        else if(strstr(step,"\"kind\":\"call\"")) {
            ++calls;
            if(strstr(step,"\"result\":\"retain\"")) {
                assert(strstr(step,"\"value\":11"));
                if(!entered){entered=1;drive(d,revision,1,0);}
                reply=value(5,throw_producer?123:99);state=throw_producer?2:1;
            } else {
                assert(strstr(step,"\"result\":\"discard\""));assert(strstr(step,"\"value\":99"));
            }
        } else {assert(!"unexpected operation");}
        free(step);
    }
    uint64_t before=*revision;PeleResult *finished=pele_public_invocation_finish_v17(d,job,revision);char *done=metadata(finished,*revision);
    assert(*revision==before);assert(strstr(done,throw_producer?"hostException":"\"Ok\""));free(done);
    failure(pele_public_invocation_finish_v17(d,job,revision),"nesting order");
    pele_public_invocation_free_v17(job);
    if(nested){assert(gets==2 && calls==0);}else{assert(gets==4 && calls==(throw_producer?1u:2u));}
}
int main(void) {
    PeleRegistry *registry=NULL; okay(pele_registry_create_v8(1,&registry));
    PelePublicConstructorV16 *factory=NULL;okay(pele_registry_capture_constructor_v16(registry,(const uint8_t *)"gantt",5,1,&factory));assert(factory);
    PeleHostValueV17 argument=value(5,10);PelePublicDocument *d=NULL;
    okay(pele_public_constructor_construct_with_v17(factory,&argument,&d));assert(d);
    pele_public_constructor_free_v16(factory);pele_registry_free_v8(registry);
    uint64_t revision=0;drive(d,&revision,0,0);drive(d,&revision,0,1);
    PeleInvocationJobV17 *cancel=prepare(d,revision);
    PeleResult *pending=pele_public_invocation_resume_v17(d,cancel,0,NULL,&revision);char *step=metadata(pending,revision);free(step);
    pele_public_invocation_free_v17(cancel);drive(d,&revision,1,0);
    /* Scalar numeric transport retains -0 and accepts lossless UTF16. */
    PeleHostValueV17 number=value(3,UINT64_C(0x8000000000000000));
    okay(pele_host_primitive_number_v17(&number));
    const uint16_t malformed[]={0xd800};number=value(4,0);number.units=malformed;number.unit_count=1;
    okay(pele_host_primitive_number_v17(&number));
    pele_public_document_free_v4(d);
    puts("C17 constructor host values, nested phases, original throw, cancellation PASS");
    return 0;
}
