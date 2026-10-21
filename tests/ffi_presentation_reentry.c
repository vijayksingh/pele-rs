#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void success(PeleResult *r) {
    assert(r && pele_result_status(r) == 0);
    pele_result_free(r);
}
static uint64_t result(PeleResult *r, const char *expected) {
    assert(r && pele_result_status(r) == 0);
    const uint8_t *bytes = pele_result_data(r);
    const size_t n = pele_result_len(r);
    assert(n >= 24 && !memcmp(bytes, "PELGBIN1", 8));
    uint64_t revision = 0;
    for (unsigned i = 0; i < 8; ++i) revision |= (uint64_t)bytes[12+i] << (8*i);
    size_t len = 0;
    for (unsigned i = 0; i < 4; ++i) len |= (size_t)bytes[20+i] << (8*i);
    assert(len <= n-24);
    char *metadata = malloc(len+1); assert(metadata);
    memcpy(metadata, bytes+n-len, len); metadata[len]=0;
    if (!strstr(metadata, expected)) fprintf(stderr,"missing %s: %s\n",expected,metadata);
    assert(strstr(metadata, expected));
    free(metadata); pele_result_free(r); return revision;
}
struct State { PelePublicDocument *document; uint64_t revision; unsigned calls; int entered, fail; };
static uint32_t width(void *opaque,const uint8_t *text,size_t length,double size,uint32_t style,
    const uint8_t *font,size_t font_len,const uint8_t *mono,size_t mono_len,double *output) {
    (void)text; (void)style; (void)font; (void)font_len; (void)mono; (void)mono_len;
    struct State *s=opaque; s->calls++;
    if(!s->entered) {
        s->entered=1;
        success(pele_public_snapshot_binary_v5(s->document));
        success(pele_public_schema_v4(s->document));
        s->revision=result(pele_public_render_presentation_v11(s->document,s->revision,
            NULL,0,NULL,3,0,NULL,NULL),"\"presentation\":");
        if(s->fail)return 1;
    }
    *output=(double)length*size*.6; return 0;
}
int main(void) {
    const char *source="info\ntitle Before";
    for(int native=0;native<2;native++) for(int fail=0;fail<2;fail++) {
        PelePublicDocument *document=NULL;
        success(pele_public_parse_v4((const uint8_t *)source,strlen(source),NULL,0,&document));
        struct State state={document,0,0,0,fail};
        PeleHostResourcesV3 host={0}; host.base.context=&state; host.base.width=width;
        PeleResult *r=native
            ? pele_public_render_native_v14(document,0,NULL,0,NULL,3,0,NULL,&host,NULL)
            : pele_public_render_presentation_v11(document,0,NULL,0,NULL,3,0,NULL,&host);
        assert(result(r,fail?"\"Err\":":"\"presentation\":")==2);
        assert(state.entered && state.calls && state.revision==1);
        unsigned calls=state.calls;
        r=pele_public_render_presentation_v11(document,0,NULL,0,NULL,3,0,NULL,&host);
        assert(r && pele_result_status(r)!=0); pele_result_free(r); assert(state.calls==calls);
        assert(result(pele_public_render_presentation_v11(document,2,NULL,0,NULL,3,0,NULL,NULL),"\"presentation\":")==3);
        pele_public_document_free_v4(document);
    }
    puts("mutable presentation/native callback reentry passed"); return 0;
}
