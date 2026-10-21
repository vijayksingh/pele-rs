#include "pele.h"
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void okay(PeleResult *r) {
    assert(r && pele_result_status(r) == 0); pele_result_free(r);
}
static void failure(PeleResult *r) {
    assert(r && pele_result_status(r) != 0); pele_result_free(r);
}
static char *json(PeleResult *r) {
    assert(r && pele_result_status(r) == 0); size_t n = pele_result_len(r);
    char *s = malloc(n + 1); assert(s); memcpy(s, pele_result_data(r), n); s[n] = 0;
    pele_result_free(r); return s;
}
static char *metadata(PeleResult *r, uint64_t revision) {
    assert(r && pele_result_status(r) == 0);
    const uint8_t *p = pele_result_data(r); size_t n = pele_result_len(r);
    assert(n >= 24 && !memcmp(p, "PELGBIN1", 8)); uint64_t actual = 0;
    for (unsigned i = 0; i < 8; ++i) actual |= (uint64_t)p[12+i] << (8*i);
    assert(actual == revision);
    size_t length = (size_t)p[20] | (size_t)p[21]<<8 | (size_t)p[22]<<16 | (size_t)p[23]<<24;
    assert(length <= n - 24); char *s = malloc(length + 1); assert(s);
    memcpy(s, p + n - length, length); s[length] = 0; pele_result_free(r); return s;
}
static PeleHostValueV17 value(uint32_t tag, uint64_t payload) {
    PeleHostValueV17 v = {0}; v.version = 17; v.struct_size = sizeof(v);
    v.tag = tag; v.payload = payload; return v;
}
static PelePublicConstructorV16 *capture(PeleRegistry *registry, const char *type, uint32_t prototype) {
    PelePublicConstructorV16 *f = NULL;
    okay(pele_registry_capture_constructor_v16(registry, (const uint8_t *)type, strlen(type), prototype, &f));
    assert(f); return f;
}
static void ordered(const char *family, const char *first_key, unsigned expected_gets, int throws) {
    PeleRegistry *registry = NULL; okay(pele_registry_create_v8(1, &registry));
    PelePublicConstructorV16 *factory = capture(registry, family, 1);
    char *policy = json(pele_public_constructor_policy_v18(factory));
    assert(strstr(policy, "\"argumentTransfers\":[\"retain\"]")); free(policy);
    PeleHostValueV17 argument = value(5, 27); PeleConstructionJobV18 *job = NULL;
    okay(pele_public_constructor_begin_v18(factory, &argument, NULL, &job)); assert(job);
    pele_public_constructor_free_v16(factory); pele_registry_free_v8(registry);
    uint64_t revision = 0; uint32_t state = 0; unsigned gets = 0;
    PeleHostValueV17 reply = value(0, 0);
    for (;;) {
        PeleResult *response = pele_public_construction_resume_v18(job, state, &reply, &revision);
        char *step = metadata(response, revision);
        if (strstr(step, "\"complete\":true")) { free(step); break; }
        assert(strstr(step, "\"kind\":\"get\""));
        if (gets++ == 0) assert(strstr(step, first_key));
        else assert(!strcmp(family, "sequence") && strstr(step, "sequenceWrap"));
        /* No native borrow spans this host phase. Reapply the exact provisional graph. */
        PeleResult *snapshot = pele_public_construction_snapshot_v18(job);
        assert(snapshot && pele_result_status(snapshot) == 0);
        okay(pele_public_construction_apply_v18(job, pele_result_data(snapshot), pele_result_len(snapshot)));
        pele_result_free(snapshot);
        reply = value(throws ? 5u : 0u, throws ? 91u : 0u); state = throws ? 2u : 1u;
        free(step);
    }
    assert(gets == (throws ? 1u : expected_gets)); PelePublicDocument *document = NULL;
    PeleResult *result = pele_public_construction_finish_v18(job, &document, &revision);
    assert(document); char *done = metadata(result, revision);
    assert(strstr(done, throws ? "hostException" : "\"Ok\"")); free(done);
    okay(pele_public_snapshot_v4(document));
    PelePublicDocument *again = document;
    failure(pele_public_construction_finish_v18(job, &again, &revision)); assert(!again);
    pele_public_construction_free_v18(job); pele_public_document_free_v4(document);
}
int main(void) {
    ordered("sequence", "\"key\":\"wrap\"", 2, 0);
    ordered("block", "\"key\":\"warn\"", 1, 0);
    ordered("agentflow", "\"key\":\"lineOffset\"", 1, 0);
    ordered("sequence", "\"key\":\"wrap\"", 2, 1);
    PeleRegistry *registry = NULL; okay(pele_registry_create_v8(1, &registry));
    PelePublicConstructorV16 *member = capture(registry, "class", 2);
    char *policy = json(pele_public_constructor_policy_v18(member));
    assert(strstr(policy, "\"argumentTransfers\":[\"value\",\"value\"]"));
    assert(strstr(policy, "\"arity\":2")); free(policy); pele_public_constructor_free_v16(member);
    PelePublicConstructorV16 *ignored = capture(registry, "pie", 1);
    PeleHostValueV17 malformed = value(99, 0); PeleConstructionJobV18 *job = NULL;
    okay(pele_public_constructor_begin_v18(ignored, &malformed, &malformed, &job)); assert(job);
    pele_public_construction_free_v18(job);
    failure(pele_public_constructor_begin_v18(ignored, NULL, NULL, NULL));
    okay(pele_public_constructor_begin_v18(ignored, NULL, NULL, &job)); uint64_t revision = 0;
    PeleResult *response = pele_public_construction_resume_v18(job, 0, NULL, &revision);
    char *step = metadata(response, revision); assert(strstr(step, "\"complete\":true")); free(step);
    PelePublicDocument *document = NULL;
    okay(pele_public_construction_finish_v18(job, &document, &revision)); assert(document);
    pele_public_document_free_v4(document); pele_public_construction_free_v18(job);
    pele_public_constructor_free_v16(ignored); pele_registry_free_v8(registry);
    puts("C18 ordered getters, partial error owner, cancellation and transfer policy PASS"); return 0;
}
