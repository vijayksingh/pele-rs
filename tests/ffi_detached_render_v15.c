#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void success(PeleResult *result) {
    assert(result && pele_result_status(result) == 0);
    pele_result_free(result);
}
static char *text(PeleResult *result) {
    const size_t length = pele_result_len(result);
    char *value = malloc(length + 1);
    assert(value);
    memcpy(value, pele_result_data(result), length);
    value[length] = 0;
    return value;
}
static uint64_t outcome(PeleResult *result, const char *message) {
    assert(result && pele_result_status(result) == 0);
    const uint8_t *bytes = pele_result_data(result);
    const size_t length = pele_result_len(result);
    assert(length >= 24 && !memcmp(bytes, "PELGBIN1", 8));
    uint64_t revision = 0;
    for (unsigned i = 0; i < 8; ++i) revision |= (uint64_t)bytes[12 + i] << (8 * i);
    const size_t size = (size_t)bytes[20] | (size_t)bytes[21] << 8 | (size_t)bytes[22] << 16 | (size_t)bytes[23] << 24;
    assert(size <= length - 24);
    char *metadata = malloc(size + 1);
    assert(metadata);
    memcpy(metadata, bytes + length - size, size);
    metadata[size] = 0;
    assert(strstr(metadata, message));
    free(metadata);
    pele_result_free(result);
    return revision;
}
static void failure(PeleResult *result, const char *message) {
    assert(result && pele_result_status(result) != 0);
    char *value = text(result);
    assert(strstr(value, message));
    free(value);
    pele_result_free(result);
}

struct Context {
    PelePublicDocument *document;
    uint64_t revision;
    unsigned widths;
    int entered;
};

static uint32_t width(void *opaque, const uint8_t *text_bytes, size_t length,
                      double size, uint32_t style, const uint8_t *font, size_t font_length,
                      const uint8_t *mono, size_t mono_length, double *value) {
    (void)text_bytes; (void)style; (void)font; (void)font_length; (void)mono; (void)mono_length;
    struct Context *context = opaque;
    context->widths++;
    if (!context->entered) {
        context->entered = 1;
        /* Derive the receiver from the public snapshot, not private arena state. */
        PeleResult *snapshot = pele_public_snapshot_v4(context->document);
        assert(snapshot && pele_result_status(snapshot) == 0);
        char *body = text(snapshot);
        const char *root = strstr(body, "\"root\":{\"kind\":\"ref\",\"value\":");
        assert(root);
        root += strlen("\"root\":{\"kind\":\"ref\",\"value\":");
        const unsigned long root_id = strtoul(root, NULL, 10);
        const char *strings = strstr(body, "\"strings\":[");
        assert(strings);
        strings += strlen("\"strings\":[");
        unsigned label_id = 0;
        while (*strings == '[' && strncmp(strings, "[65]", 4)) {
            strings = strchr(strings, ']');
            assert(strings);
            ++strings;
            if (*strings == ',') ++strings;
            ++label_id;
        }
        assert(!strncmp(strings, "[65]", 4));
        free(body);
        pele_result_free(snapshot);
        success(pele_public_schema_v4(context->document));

        /* A helper invoked on its own existing node must return that alias
           during the outer render. String and object IDs come from the snapshot. */
        char call[256];
        const int count = snprintf(call, sizeof(call),
            "{\"method\":1,\"receiver\":{\"kind\":\"ref\",\"value\":%lu},\"arguments\":[{\"kind\":\"string\",\"value\":%u}]}", root_id, label_id);
        assert(count > 0 && (size_t)count < sizeof(call));
        context->revision = outcome(pele_public_invoke_with_functions_v12(context->document,
            context->revision, (const uint8_t *)call, (size_t)count, NULL), "\"Ok\":{\"kind\":\"ref\"");

        PeleDetachedRenderJobV15 *nested = NULL;
        success(pele_public_prepare_detached_render_v15(context->document, context->revision,
            NULL, 0, NULL, 3, 0, &nested));
        assert(nested);
        success(pele_detached_render_run_v15(nested, NULL));
        const uint64_t revision = outcome(pele_public_finish_detached_render_v15(
            context->document, nested, &context->revision), "\"Ok\":");
        assert(revision == context->revision);
        pele_detached_render_job_free_v15(nested);
    }
    *value = (double)length * size * 0.6;
    return 0;
}

static uint32_t flow_width(void *opaque,const uint8_t *bytes,size_t length,double size,uint32_t style,const uint8_t *font,size_t font_length,const uint8_t *mono,size_t mono_length,double *value){
    (void)bytes;(void)style;(void)font;(void)font_length;(void)mono;(void)mono_length;
    struct Context *context=opaque;context->widths++;
    if(!context->entered){
        context->entered=1;success(pele_public_snapshot_v4(context->document));
        PeleDetachedRenderJobV15 *nested=NULL;
        success(pele_public_prepare_detached_render_v15(context->document,context->revision,NULL,0,NULL,3,0,&nested));assert(nested);
        success(pele_detached_render_run_v15(nested,NULL));
        outcome(pele_public_finish_detached_render_v15(context->document,nested,&context->revision),"\"Ok\":");
        pele_detached_render_job_free_v15(nested);
    }
    *value=(double)length*size*.6;return 0;
}
static void flow_array_consumers(void){
    const char *sources[]={"flowchart LR\nA[Alpha] --> B[Beta]\nstyle B fill:#113355\n", "swimlane-beta LR\nsubgraph Lane\nA[Alpha] --> B[Beta]\nend\nstyle B fill:#113355\n"};
    for(size_t i=0;i<sizeof(sources)/sizeof(sources[0]);i++){
        PelePublicDocument *document=NULL;success(pele_public_parse_v4((const uint8_t *)sources[i],strlen(sources[i]),NULL,0,&document));
        struct Context context={document,0,0,0};PeleHostResourcesV3 resources={0};resources.base.context=&context;resources.base.width=flow_width;PeleDetachedRenderJobV15 *job=NULL;
        success(pele_public_prepare_detached_render_v15(document,0,NULL,0,NULL,3,0,&job));assert(job);success(pele_detached_render_run_v15(job,&resources));
        assert(context.entered&&context.widths>0&&context.revision==1);
        assert(outcome(pele_public_finish_detached_render_v15(document,job,&context.revision),"\"Ok\":")==2);
        pele_detached_render_job_free_v15(job);pele_public_document_free_v4(document);
    }
}

int main(void) {
    const char *source = "sankey\nA,B,5\nB,C,3";
    PelePublicDocument *document = NULL;
    success(pele_public_parse_v4((const uint8_t *)source, strlen(source), NULL, 0, &document));
    struct Context context = {document, 0, 0, 0};
    PeleHostResourcesV3 resources = {0};
    resources.base.context = &context;
    resources.base.width = width;
    PeleDetachedRenderJobV15 *job = NULL;
    success(pele_public_prepare_detached_render_v15(document, 0, NULL, 0, NULL, 3, 0, &job));
    assert(job);
    success(pele_detached_render_run_v15(job, &resources));
    assert(context.entered && context.widths > 0 && context.revision == 2);
    assert(outcome(pele_public_finish_detached_render_v15(document, job, &context.revision), "\"Ok\":") == 3);
    assert(context.revision == 3);
    failure(pele_detached_render_run_v15(job, NULL), "already dispatched");
    failure(pele_public_finish_detached_render_v15(document, job, &context.revision), "no completion");
    pele_detached_render_job_free_v15(job);

    /* Cancelling an unrun job releases its reservation; ordinary rendering and
       a later job remain available on the same public owner. */
    success(pele_public_prepare_detached_render_v15(document, context.revision,
        NULL, 0, NULL, 3, 0, &job));
    assert(job);
    pele_detached_render_job_free_v15(job);
    context.revision = outcome(pele_public_render_operation_with_numbers_v9(document,
        context.revision, NULL, 0, NULL, 3, 0, NULL), "\"Ok\":");
    assert(context.revision == 4);
    pele_public_document_free_v4(document);

    PelePublicDocument *pie = NULL;
    source = "pie\n\"A\":1";
    success(pele_public_parse_v4((const uint8_t *)source, strlen(source), NULL, 0, &pie));
    job = NULL;
    success(pele_public_prepare_detached_render_v15(pie, 0, NULL, 0, NULL, 3, 0, &job));
    /* Pie now exposes the same owned lifecycle; it no longer falls back. */
    assert(job);
    success(pele_detached_render_run_v15(job, NULL));
    uint64_t pie_revision = 0;
    assert(outcome(pele_public_finish_detached_render_v15(pie, job, &pie_revision), "\"Ok\":") == 1);
    assert(pie_revision == 1);
    pele_detached_render_job_free_v15(job);
    assert(outcome(pele_public_render_operation_with_numbers_v9(pie, pie_revision, NULL, 0, NULL, 3, 0, NULL), "\"Ok\":") == 2);
    pele_public_document_free_v4(pie);
    flow_array_consumers();
    puts("native detached render lifecycle passed");
    return 0;
}
