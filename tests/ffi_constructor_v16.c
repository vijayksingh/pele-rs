#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char *text(PeleResult *result) {
    assert(result);
    size_t n = pele_result_len(result);
    char *out = malloc(n + 1); assert(out);
    memcpy(out, pele_result_data(result), n); out[n] = 0; return out;
}
static void success(PeleResult *result, const char *needle) {
    assert(result && pele_result_status(result) == 0);
    char *body = text(result); if (needle) assert(strstr(body, needle));
    free(body); pele_result_free(result);
}
static void failure(PeleResult *result) {
    assert(result && pele_result_status(result) != 0); pele_result_free(result);
}
static uint64_t le64(const uint8_t *p) {
    uint64_t out = 0; for (unsigned i = 0; i < 8; ++i) out |= (uint64_t)p[i] << (i * 8); return out;
}
static void rendered(PeleResult *result, uint64_t revision) {
    assert(result && pele_result_status(result) == 0);
    const uint8_t *bytes = pele_result_data(result); size_t len = pele_result_len(result);
    assert(len >= 24 && !memcmp(bytes, "PELGBIN1", 8)); assert(le64(bytes + 12) == revision);
    size_t n = (size_t)bytes[20] | (size_t)bytes[21] << 8 | (size_t)bytes[22] << 16 | (size_t)bytes[23] << 24;
    assert(n <= len - 24); char *meta = malloc(n + 1); assert(meta);
    memcpy(meta, bytes + len - n, n); meta[n] = 0;
    assert(strstr(meta, "\"Ok\":")); assert(strstr(meta, "pele-sankey"));
    free(meta); pele_result_free(result);
}
static PeleResult *capture(PeleRegistry *registry, const char *name, uint32_t prototype,
                           PelePublicConstructorV16 **out) {
    return pele_registry_capture_constructor_v16(registry, (const uint8_t *)name, strlen(name), prototype, out);
}
static void additional_constructors(void) {
    const char *families[] = {"venn", "ishikawa", "requirement", "railroad", "c4", "xychart", "er",
        "architecture", "kanban", "mindmap", "timeline", "journey", "quadrantChart", "state"};
    const size_t count = sizeof families / sizeof families[0];
    PelePublicConstructorV16 *factories[sizeof families / sizeof families[0]] = {0};
    PeleRegistry *registry = NULL;
    success(pele_registry_create_v8(1, &registry), NULL);
    for (size_t i = 0; i < count; ++i) {
        success(capture(registry, families[i], 1, &factories[i]), "\"selected\":true");
        assert(factories[i]);
    }
    pele_registry_free_v8(registry);
    for (size_t i = 0; i < count; ++i) {
        PelePublicDocument *document = NULL;
        char expected[64];
        int length = snprintf(expected, sizeof expected, "\"type\":\"%s\"", families[i]);
        assert(length > 0 && (size_t)length < sizeof expected);
        success(pele_public_constructor_construct_v16(factories[i], &document), expected);
        assert(document);
        pele_public_constructor_free_v16(factories[i]);
        success(pele_public_snapshot_v4(document), "\"revision\":\"0\"");
        PeleResult *result = pele_render_prepared_v13(document, 0, NULL, 0, NULL, 0, NULL, NULL);
        assert(result && pele_result_status(result) == 0);
        const uint8_t *bytes = pele_result_data(result);
        size_t size = pele_result_len(result);
        assert(size >= 24 && !memcmp(bytes, "PELGBIN1", 8) && le64(bytes + 12) == 1);
        size_t metadata_size = (size_t)bytes[20] | (size_t)bytes[21] << 8 |
            (size_t)bytes[22] << 16 | (size_t)bytes[23] << 24;
        assert(metadata_size <= size - 24);
        char *metadata = malloc(metadata_size + 1); assert(metadata);
        memcpy(metadata, bytes + size - metadata_size, metadata_size); metadata[metadata_size] = 0;
        if (!strcmp(families[i], "xychart")) {
            assert(strstr(metadata, "\"Err\":"));
            assert(strstr(metadata, "No Plot to render, please provide a plot with some data"));
        } else {
            assert(strstr(metadata, "\"Ok\":"));
            assert(strstr(metadata, "<svg"));
        }
        free(metadata); pele_result_free(result);
        pele_public_document_free_v4(document);
    }
}
int main(void) {
    PeleRegistry *registry = NULL;
    success(pele_registry_create_v8(1, &registry), NULL); assert(registry);
    PelePublicConstructorV16 *factory = NULL, *absent = (PelePublicConstructorV16 *)(uintptr_t)1;
    success(capture(registry, "pie", 1, &absent), "\"selected\":false"); assert(!absent);
    success(capture(registry, "sankey", 999, &absent), "\"selected\":false"); assert(!absent);
    failure(capture(registry, "unknown-family", 1, &absent)); assert(!absent);
    success(capture(registry, "sankey", 1, &factory), "\"selected\":true"); assert(factory);
    /* Captured registration outlives replacement and registry destruction. */
    success(pele_registry_register_builtin_v8(registry, (const uint8_t *)"sankey", 6), NULL);
    pele_registry_free_v8(registry);
    PelePublicDocument *a = NULL, *b = NULL;
    success(pele_public_constructor_construct_v16(factory, &a), "\"constructorCreated\":true");
    success(pele_public_constructor_construct_v16(factory, &b), "\"constructorCreated\":true");
    assert(a && b && a != b);
    pele_public_constructor_free_v16(factory);
    PeleResult *left = pele_public_snapshot_v4(a), *right = pele_public_snapshot_v4(b);
    assert(pele_result_status(left) == 0 && pele_result_status(right) == 0);
    assert(pele_result_len(left) == pele_result_len(right));
    assert(!memcmp(pele_result_data(left), pele_result_data(right), pele_result_len(left)));
    char *body = text(left);
    const char *root = strstr(body, "\"root\":{\"kind\":\"ref\",\"value\":"); assert(root);
    unsigned long root_id = strtoul(root + strlen("\"root\":{\"kind\":\"ref\",\"value\":"), NULL, 10);
    const char *strings = strstr(body, "\"strings\":["); assert(strings); strings += strlen("\"strings\":[");
    unsigned label_id = 0;
    while (*strings == '[' && strncmp(strings, "[115,97,110,107,101,121]", 22)) {
        strings = strchr(strings, ']'); assert(strings); ++strings; if (*strings == ',') ++strings; ++label_id;
    }
    assert(!strncmp(strings, "[115,97,110,107,101,121]", 22));
    free(body); pele_result_free(left);
    success(pele_public_schema_v4(a), "findOrCreateNode");
    rendered(pele_render_prepared_v13(a, 0, NULL, 0, NULL, 0, NULL, NULL), 1);
    char call[256]; int n = snprintf(call, sizeof call,
        "{\"method\":1,\"receiver\":{\"kind\":\"ref\",\"value\":%lu},\"arguments\":[{\"kind\":\"string\",\"value\":%u}]}", root_id, label_id);
    assert(n > 0 && (size_t)n < sizeof call);
    success(pele_public_invoke_v4(a, 1, (const uint8_t *)call, (size_t)n, NULL), "\"kind\":\"ref\"");
    rendered(pele_public_render_operation_with_numbers_v9(a, 2, NULL, 0, NULL, 3, 0, NULL), 3);
    /* Sibling still exactly initial, even after first owner's helper and render. */
    PeleResult *unchanged = pele_public_snapshot_v4(b);
    assert(pele_result_status(unchanged) == 0 && pele_result_len(unchanged) == pele_result_len(right));
    assert(!memcmp(pele_result_data(unchanged), pele_result_data(right), pele_result_len(right)));
    pele_result_free(unchanged); pele_result_free(right);
    pele_public_document_free_v4(a); pele_public_document_free_v4(b);
    pele_public_constructor_free_v16(NULL);
    additional_constructors();
    puts("C16 constructor lifecycle passed: capture, independent defaults, helper, raw/normal render, disposal");
    return 0;
}
