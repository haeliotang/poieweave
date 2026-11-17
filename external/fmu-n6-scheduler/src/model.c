/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Parameterized independent scheduler for up to three tasks and two robots.
 * No Poieweave scheduler/engine linkage.
 */
#include "fmi3.h"
#include <math.h>
#include <stdlib.h>

#define MAXT 3
#define MAXR 2
#define VR_TASK_COUNT 2
#define VR_T1_DUR 3
#define VR_T1_PRIO 4
#define VR_T1_MASK 5
#define VR_T1_TOOL 6
#define VR_T2_DUR 7
#define VR_T2_PRIO 8
#define VR_T2_MASK 9
#define VR_T2_TOOL 10
#define VR_T3_DUR 11
#define VR_T3_PRIO 12
#define VR_T3_MASK 13
#define VR_T3_TOOL 14
#define VR_ROBOT_COUNT 15
#define VR_FAIL_TIME 16
#define VR_FAIL_KIND 17
#define VR_FAIL_INDEX 18
#define VR_HORIZON 19
#define VR_TIME 1
#define VR_MAKESPAN 20
#define VR_COMPLETED 21
#define VR_INTERRUPTED 22
#define VR_FAIL_APPLIED 23
#define VR_T1_START 24
#define VR_T1_END 25
#define VR_T1_ROBOT 26
#define VR_T2_START 27
#define VR_T2_END 28
#define VR_T2_ROBOT 29
#define VR_T3_START 30
#define VR_T3_END 31
#define VR_T3_ROBOT 32
#define VR_REST1 33
#define VR_REST2 34
#define VR_REST3 35
#define VR_LAST_EVENT 36
#define VR_TOOL_AVAILABLE 37
#define VR_TOOL_FAIL_APPLIED 38
#define VR_RELEASE 39

typedef struct {
    fmi3Float64 time;
    fmi3Float64 task_count, robot_count, fail_time, fail_kind, fail_index, horizon;
    fmi3Float64 dur[MAXT], prio[MAXT], mask[MAXT], tool[MAXT];
    fmi3Float64 start[MAXT], end[MAXT], robot[MAXT];
    fmi3Float64 makespan, completed, interrupted, fail_applied, release;
    fmi3Float64 restored[MAXT];
    fmi3Float64 last_event, tool_available, tool_fail_applied;
} Component;

const char *fmi3GetVersion(void) { return "3.0"; }
fmi3Status fmi3SetDebugLogging(fmi3Instance i, fmi3Boolean b, size_t n, const char *c[]) {
    (void)i; (void)b; (void)n; (void)c; return fmi3OK;
}
fmi3Instance fmi3InstantiateCoSimulation(const char *a, const char *b, const char *c, fmi3Boolean d, fmi3Boolean e,
                                         fmi3Boolean f, fmi3Boolean g, const char *const *h, size_t i,
                                         fmi3InstanceEnvironment j, fmi3LogMessageCallback k,
                                         fmi3IntermediateUpdateCallback l) {
    (void)a; (void)b; (void)c; (void)d; (void)e; (void)f; (void)g; (void)h; (void)i; (void)j; (void)k; (void)l;
    Component *x = (Component *)calloc(1, sizeof(Component));
    if (!x) return NULL;
    x->task_count = 2; x->robot_count = 1; x->fail_time = -1; x->horizon = 72000;
    x->tool_available = 1;
    return x;
}
fmi3Status fmi3EnterInitializationMode(fmi3Instance i, fmi3Boolean a, fmi3Float64 b, fmi3Float64 c, fmi3Boolean d,
                                       fmi3Float64 e) {
    (void)a; (void)b; (void)d; (void)e;
    Component *x = (Component *)i;
    if (!x) return fmi3Error;
    x->time = c;
    int n = (int)x->task_count; if (n < 0) n = 0; if (n > MAXT) n = MAXT;
    int robots = (int)x->robot_count; if (robots < 1) robots = 1; if (robots > MAXR) robots = MAXR;
    fmi3Float64 rob_avail[MAXR] = {0.0, 0.0}, tool_avail = 0.0;
    int order[MAXT] = {0, 1, 2};
    for (int a_i = 0; a_i < n; ++a_i)
        for (int b_i = a_i + 1; b_i < n; ++b_i)
            if (x->prio[order[b_i]] < x->prio[order[a_i]]) { int t = order[a_i]; order[a_i] = order[b_i]; order[b_i] = t; }
    x->makespan = 0.0;
    for (int k = 0; k < n; ++k) {
        int t = order[k];
        fmi3Float64 start = 0.0;
        int chosen = -1;
        for (int r = 0; r < robots; ++r) {
            if ((int)x->mask[t] & (1 << r)) {
                if (chosen < 0 || rob_avail[r] < rob_avail[chosen]) chosen = r;
            }
        }
        if (chosen < 0) { x->interrupted = 1; x->last_event = 6; return fmi3OK; }
        start = rob_avail[chosen];
        if (x->tool[t] > 0.5 && tool_avail > start) start = tool_avail;
        x->start[t] = start;
        x->end[t] = start + x->dur[t];
        x->robot[t] = chosen + 1;
        rob_avail[chosen] = x->end[t];
        if (x->tool[t] > 0.5) tool_avail = x->end[t];
        if (x->end[t] > x->makespan) x->makespan = x->end[t];
    }
    x->completed = 0; x->interrupted = 0; x->fail_applied = -1; x->release = x->makespan;
    for (int t = 0; t < MAXT; ++t) x->restored[t] = 0;
    return fmi3OK;
}
fmi3Status fmi3ExitInitializationMode(fmi3Instance i) { return i ? fmi3OK : fmi3Error; }
fmi3Status fmi3SetFloat64(fmi3Instance i, const fmi3ValueReference vr[], size_t n, const fmi3Float64 v[], size_t m) {
    Component *x = (Component *)i;
    if (!x || n != m) return fmi3Error;
    for (size_t k = 0; k < n; ++k) {
        switch (vr[k]) {
        case VR_TASK_COUNT: x->task_count = v[k]; break;
        case VR_T1_DUR: x->dur[0] = v[k]; break; case VR_T1_PRIO: x->prio[0] = v[k]; break;
        case VR_T1_MASK: x->mask[0] = v[k]; break; case VR_T1_TOOL: x->tool[0] = v[k]; break;
        case VR_T2_DUR: x->dur[1] = v[k]; break; case VR_T2_PRIO: x->prio[1] = v[k]; break;
        case VR_T2_MASK: x->mask[1] = v[k]; break; case VR_T2_TOOL: x->tool[1] = v[k]; break;
        case VR_T3_DUR: x->dur[2] = v[k]; break; case VR_T3_PRIO: x->prio[2] = v[k]; break;
        case VR_T3_MASK: x->mask[2] = v[k]; break; case VR_T3_TOOL: x->tool[2] = v[k]; break;
        case VR_ROBOT_COUNT: x->robot_count = v[k]; break;
        case VR_FAIL_TIME: x->fail_time = v[k]; break; case VR_FAIL_KIND: x->fail_kind = v[k]; break;
        case VR_FAIL_INDEX: x->fail_index = v[k]; break; case VR_HORIZON: x->horizon = v[k]; break;
        default: break;
        }
    }
    return fmi3OK;
}
static int selected_robot(Component *x, int robot) {
    for (int t = 0; t < MAXT; ++t) if ((int)x->robot[t] == robot) return 1;
    return 0;
}
static int uses_tool(Component *x) { for (int t = 0; t < MAXT; ++t) if (x->tool[t] > 0.5) return 1; return 0; }
fmi3Status fmi3GetFloat64(fmi3Instance i, const fmi3ValueReference vr[], size_t n, fmi3Float64 v[], size_t m) {
    Component *x = (Component *)i;
    if (!x || n != m) return fmi3Error;
    for (size_t k = 0; k < n; ++k) {
        switch (vr[k]) {
        case VR_TIME: v[k] = x->time; break; case VR_MAKESPAN: v[k] = x->makespan; break;
        case VR_COMPLETED: v[k] = x->completed; break; case VR_INTERRUPTED: v[k] = x->interrupted; break;
        case VR_FAIL_APPLIED: v[k] = x->fail_applied; break;
        case VR_T1_START: v[k] = x->start[0]; break; case VR_T1_END: v[k] = x->end[0]; break; case VR_T1_ROBOT: v[k] = x->robot[0]; break;
        case VR_T2_START: v[k] = x->start[1]; break; case VR_T2_END: v[k] = x->end[1]; break; case VR_T2_ROBOT: v[k] = x->robot[1]; break;
        case VR_T3_START: v[k] = x->start[2]; break; case VR_T3_END: v[k] = x->end[2]; break; case VR_T3_ROBOT: v[k] = x->robot[2]; break;
        case VR_REST1: v[k] = x->restored[0]; break; case VR_REST2: v[k] = x->restored[1]; break; case VR_REST3: v[k] = x->restored[2]; break;
        case VR_LAST_EVENT: v[k] = x->last_event; break; case VR_TOOL_AVAILABLE: v[k] = x->tool_available; break;
        case VR_TOOL_FAIL_APPLIED: v[k] = x->tool_fail_applied; break; case VR_RELEASE: v[k] = x->release; break;
        default: v[k] = 0; break;
        }
    }
    return fmi3OK;
}
fmi3Status fmi3DoStep(fmi3Instance i, fmi3Float64 current, fmi3Float64 step, fmi3Boolean a, fmi3Boolean *eh,
                      fmi3Boolean *term, fmi3Boolean *early, fmi3Float64 *last) {
    (void)a;
    Component *x = (Component *)i;
    if (!x) return fmi3Error;
    if (eh) *eh = true; if (term) *term = false; if (early) *early = false; if (last) *last = x->time;
    if (fabs(current - x->time) > 1e-9) return fmi3Error;
    fmi3Float64 remaining = step;
    while (remaining > 1e-12) {
        fmi3Float64 target = x->time + remaining;
        fmi3Float64 boundary = fmin(target, x->horizon);
        if (x->fail_time >= 0 && x->fail_time > x->time + 1e-9 && x->fail_time < boundary) boundary = x->fail_time;
        if (x->completed < 0.5 && x->makespan > x->time + 1e-9 && x->makespan < boundary) boundary = x->makespan;
        fmi3Float64 piece = boundary - x->time;
        x->time += piece; remaining -= piece;
        if (x->fail_time >= 0 && x->fail_applied < 0 && x->time >= x->fail_time - 1e-9) {
            x->fail_applied = x->fail_time; x->last_event = 4;
            int hit = 0;
            if ((int)x->fail_kind == 1) hit = selected_robot(x, (int)x->fail_index);
            else if ((int)x->fail_kind == 2) { x->tool_available = 0; x->tool_fail_applied = x->fail_time; hit = uses_tool(x); }
            if (hit && x->fail_time < x->makespan - 1e-9) {
                x->interrupted = 1; x->completed = 0;
                for (int t = 0; t < MAXT; ++t) x->restored[t] = 0;
                x->release = x->fail_time;
                if (term) *term = true; if (last) *last = x->fail_time;
                return fmi3OK;
            }
        }
        if (x->completed < 0.5 && x->time >= x->makespan - 1e-9) {
            int n = (int)x->task_count; if (n < 0) n = 0; if (n > MAXT) n = MAXT;
            x->completed = n;
            for (int t = 0; t < n; ++t) x->restored[t] = 1;
            x->release = x->makespan; x->last_event = 2;
        }
        if (x->time >= x->horizon - 1e-9) {
            x->last_event = 5;
            if (term) *term = true; if (last) *last = x->horizon;
            return fmi3OK;
        }
    }
    return fmi3OK;
}
fmi3Status fmi3Terminate(fmi3Instance i) { (void)i; return fmi3OK; }
fmi3Status fmi3FreeInstance(fmi3Instance i) { free(i); return fmi3OK; }
fmi3Status fmi3Reset(fmi3Instance i) { (void)i; return fmi3OK; }
