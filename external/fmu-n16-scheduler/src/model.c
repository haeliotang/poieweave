/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Parameterized independent scheduler for up to three tasks and two robots.
 * No Poieweave scheduler/engine linkage.
 */
#include "fmi3.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAXT 3
#ifndef JOURNAL_CAPACITY
#define JOURNAL_CAPACITY 32
#endif
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
#define VR_TOOL_ACQUIRED 40
#define VR_TOOL_RELEASED 41
#define VR_TOOL_MASK 42
#define VR_PREFERRED 43
#define VR_FAIL_ASSET 44
#define VR_DEP1 45
#define VR_DEP2 46
#define VR_DEP_MASK 47
#define VR_FAIL_TARGET 48
#define VR_DEP_PATH 49
#define VR_AFFECTED 50
#define VR_FAIL_COUNT 60
#define VR_F2_TIME 61
#define VR_F2_KIND 62
#define VR_F2_INDEX 63
#define VR_F2_ASSET 64
#define VR_F3_TIME 65
#define VR_F3_KIND 66
#define VR_F3_INDEX 67
#define VR_F3_ASSET 68
#define VR_F4_TIME 69
#define VR_F4_KIND 70
#define VR_F4_INDEX 71
#define VR_F4_ASSET 72
#define VR_RELEASE_MODE 73

typedef struct {
    fmi3Float64 time;
    fmi3Float64 task_count, robot_count, fail_time, fail_kind, fail_index, horizon;
    fmi3Float64 dur[MAXT], prio[MAXT], mask[MAXT], tool[MAXT];
    fmi3Float64 start[MAXT], end[MAXT], robot[MAXT];
    fmi3Float64 makespan, completed, interrupted, fail_applied, release;
    fmi3Float64 restored[MAXT];
    fmi3Float64 last_event, tool_available, tool_fail_applied;
    fmi3Float64 tool_acquired, tool_released, tool_mask;
    fmi3Float64 preferred_robot, fail_asset, dep1, dep2;
    fmi3Float64 dep_mask, fail_target, dep_path, affected;
    fmi3Float64 journal_count, journal_overflow, journal_terminated;
    fmi3Float64 failure_target_id;
    char failure_target_name[32];
    char journal[65536];
    char journal_output[65536];
    unsigned long long failed_mask, journal_before_mask, journal_after_mask;
    fmi3Float64 fail_count;
    fmi3Float64 f_time[4], f_kind[4], f_index[4], f_asset[4], f_applied[4];
    int journal_failure_id;
    fmi3Float64 release_mode, task_done[3], resource_released[8];
    int journal_dep_before, journal_dep_after;
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
    x->fail_asset = -1; x->dep_mask = 0; x->fail_target = -1; x->dep_path = 0; x->affected = 0;
    return x;
}
static void record_transition(Component *x, const char *kind, const char *target,
                              int rb, int hb, int db, int fb, int pb, int sb, int tb);
static void apply_completion(Component *x);
static void record_termination(Component *x);

static void resolve_failure_target(Component *x);

static const char *asset_name(int id);

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
        int preferred = (int)x->preferred_robot;
        if (preferred > 0 && preferred <= robots && ((int)x->mask[t] & (1 << (preferred - 1)))) {
            chosen = preferred - 1;
        } else {
            for (int r = 0; r < robots; ++r) {
                if ((int)x->mask[t] & (1 << r)) {
                    if (chosen < 0 || rob_avail[r] < rob_avail[chosen]) chosen = r;
                }
            }
        }
        if (chosen < 0) { x->interrupted = 1; x->last_event = 6; return fmi3OK; }
        start = rob_avail[chosen];
        if (x->tool[t] > 0.5 && tool_avail > start) start = tool_avail;
        x->start[t] = start;
        x->end[t] = start + x->dur[t];
        x->robot[t] = chosen + 1;
        x->affected = chosen + 1;
        rob_avail[chosen] = x->end[t];
        if (x->tool[t] > 0.5) tool_avail = x->end[t];
        if (x->end[t] > x->makespan) x->makespan = x->end[t];
    }
    x->completed = 0; x->interrupted = 0; x->fail_applied = -1; x->release = x->makespan;
    for (int t = 0; t < MAXT; ++t) x->restored[t] = 0;
    x->tool_acquired = -1; x->tool_released = -1; x->tool_mask = 0;
    for (int t = 0; t < n; ++t) {
        if (x->tool[t] > 0.5) {
            x->tool_mask += (fmi3Float64)(1 << t);
            if (x->tool_acquired < 0 || x->start[t] < x->tool_acquired) x->tool_acquired = x->start[t];
        }
    }
    if (x->tool_mask > 0) x->tool_released = x->makespan;
    x->dep_mask = 0;
    for (int t = 0; t < n; ++t) {
        if ((int)x->robot[t] == 1) x->dep_mask = (fmi3Float64)(((int)x->dep_mask) | ((int)x->dep1));
        if ((int)x->robot[t] == 2) x->dep_mask = (fmi3Float64)(((int)x->dep_mask) | ((int)x->dep2));
    }
    x->f_time[0] = x->fail_time; x->f_kind[0] = x->fail_kind;
    x->f_index[0] = x->fail_index; x->f_asset[0] = x->fail_asset;
    if (x->fail_count < 0 || x->fail_count > 4 || x->fail_count != (int)x->fail_count) return fmi3Error;
    for (int fi = 0; fi < 4; ++fi) { if (fi >= (int)x->fail_count) { x->f_time[fi] = -1; x->f_applied[fi] = 1; } }
    for (int fi = 0; fi < (int)x->fail_count; ++fi) {
        if (x->f_time[fi] < 0 || !isfinite(x->f_time[fi])) return fmi3Error;
        int k = (int)x->f_kind[fi];
        if (k < 1 || k > 3) return fmi3Error;
        if (k == 3 && ((int)x->f_asset[fi] < 0 || (int)x->f_asset[fi] > 7)) return fmi3Error;
        if (k == 1 && ((int)x->f_index[fi] < 1 || (int)x->f_index[fi] > 2)) return fmi3Error;
    }
    resolve_failure_target(x);
    x->failed_mask = 0; x->journal_before_mask = 0; x->journal_after_mask = 0;
    x->journal[0] = 0; x->journal_count = 0; x->journal_overflow = 0; x->journal_terminated = 0;
    x->journal_dep_before = 0; x->journal_dep_after = (int)x->dep_mask;
    record_transition(x, "InitialRepairStarted", "", 0, 1, 0, 0, 0, 0, 0);
    return fmi3OK;
}
fmi3Status fmi3ExitInitializationMode(fmi3Instance i) { return i ? fmi3OK : fmi3Error; }

static const char *asset_name(int id) {
    static const char *names[] = {"robot-m1","robot-m2","charger-1","bus-1","bat-1","sensor-x","bus-2","tool-t1"};
    if (id < 0 || id > 7) return "unknown";
    return names[id];
}
static void resolve_failure_target_id(Component *x, int which) {
    int kind = (int)x->f_kind[which];
    int id = -1;
    if (kind == 1) id = (int)x->f_index[which] - 1;
    else if (kind == 2) id = 7;
    else if (kind == 3) id = (int)x->f_asset[which];
    x->failure_target_id = id;
    snprintf(x->failure_target_name, sizeof(x->failure_target_name), "%s", asset_name(id));
}
static void resolve_failure_target(Component *x) { resolve_failure_target_id(x, 0); }
static void append_event(Component *x, const char *kind, const char *target,
                         int rb, int hb, int db, int fb, int pb, int sb, int tb,
                         int ra, int ha, int da, int fa, int pa, int sa, int ta) {
    if (x->journal_count >= JOURNAL_CAPACITY) { x->journal_overflow = 1; return; }
    x->journal_count += 1;
    int n = (int)x->journal_count;
    size_t used = strlen(x->journal);
    snprintf(x->journal + used, sizeof(x->journal) - used,
             "%s{\"seq\":%d,\"at\":%.9f,\"kind\":\"%s\",\"origin\":\"NATIVE_EXTERNAL_FMU\",\"target\":\"%s\","
             "\"target_id\":%d,\"failure_id\":%d,\"task_done\":%d,\"released_mask\":%d,\"dep_before\":%d,\"dep_after\":%d,\"failed_before\":%llu,\"failed_after\":%llu,\"before\":[%d,%d,%d,%d,%d,%d,%d],\"after\":[%d,%d,%d,%d,%d,%d,%d]}",
             n == 1 ? "" : ",", n, x->time, kind, target, (int)x->failure_target_id, x->journal_failure_id,
             (int)((x->task_done[0] > 0.5) | ((x->task_done[1] > 0.5) << 1) | ((x->task_done[2] > 0.5) << 2)),
             (int)x->resource_released[0],
             x->journal_dep_before, x->journal_dep_after,
             x->journal_before_mask, x->journal_after_mask,
             rb, hb, db, fb, pb, sb, tb, ra, ha, da, fa, pa, sa, ta);
}
static void event_states(Component *x, int *robot, int *health, int *dep, int *fail, int *repair, int *rest, int *term) {
    *robot = (int)x->affected; *health = (int)x->tool_available;
    *dep = (int)x->dep_mask; *fail = x->fail_applied >= 0 ? 1 : 0;
    *repair = x->interrupted > 0.5 ? 2 : (x->completed > 0.5 ? 1 : 0);
    *rest = x->restored[0] > 0.5 ? 1 : 0; *term = x->journal_terminated > 0.5 ? 1 : 0;
}
static void record_transition(Component *x, const char *kind, const char *target,
                              int rb, int hb, int db, int fb, int pb, int sb, int tb) {
    if (kind[0] != 'A' || kind[1] != 's') { x->journal_before_mask = x->failed_mask; x->journal_after_mask = x->failed_mask; }
    int ra, ha, da, fa, pa, sa, ta;
    event_states(x, &ra, &ha, &da, &fa, &pa, &sa, &ta);
    append_event(x, kind, target, rb, hb, db, fb, pb, sb, tb, ra, ha, da, fa, pa, sa, ta);
}
static void apply_completion(Component *x) {
    int r, h, d, f, p, s, t;
    event_states(x, &r, &h, &d, &f, &p, &s, &t);
    int n = (int)x->task_count; if (n < 0) n = 0; if (n > MAXT) n = MAXT;
    x->completed = n;
    x->journal_before_mask = x->failed_mask; x->journal_after_mask = x->failed_mask;
    int r2, h2, d2, f2, p2, s2, t2;
    event_states(x, &r2, &h2, &d2, &f2, &p2, &s2, &t2);
    append_event(x, "RepairCompleted", "a-lathe", r,h,d,f,p,s,t, r2,h2,d2,f2,p2,s2,t2);
    int r3, h3, d3, f3, p3, s3, t3;
    event_states(x, &r3, &h3, &d3, &f3, &p3, &s3, &t3);
    x->journal_before_mask = x->failed_mask; x->journal_after_mask = x->failed_mask;
    for (int i = 0; i < MAXT; ++i) x->restored[i] = 1;
    x->release = x->makespan;
    if (x->tool_mask > 0) x->tool_released = x->makespan;
    int r4, h4, d4, f4, p4, s4, t4;
    event_states(x, &r4, &h4, &d4, &f4, &p4, &s4, &t4);
    append_event(x, "RestorationCommitted", "a-lathe", r3,h3,d3,f3,p3,s3,t3, r4,h4,d4,f4,p4,s4,t4);
}
static void record_termination(Component *x) {
    if (x->journal_terminated > 0.5) return;
    int r, h, d, f, p, s, t;
    event_states(x, &r, &h, &d, &f, &p, &s, &t);
    x->journal_terminated = 1;
    record_transition(x, "RunTerminated", "", r, h, d, f, p, s, t);
}

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
        case VR_FAIL_COUNT: x->fail_count = v[k]; break;
        case VR_F2_TIME: x->f_time[1] = v[k]; break; case VR_F2_KIND: x->f_kind[1] = v[k]; break;
        case VR_F2_INDEX: x->f_index[1] = v[k]; break; case VR_F2_ASSET: x->f_asset[1] = v[k]; break;
        case VR_F3_TIME: x->f_time[2] = v[k]; break; case VR_F3_KIND: x->f_kind[2] = v[k]; break;
        case VR_F3_INDEX: x->f_index[2] = v[k]; break; case VR_F3_ASSET: x->f_asset[2] = v[k]; break;
        case VR_F4_TIME: x->f_time[3] = v[k]; break; case VR_F4_KIND: x->f_kind[3] = v[k]; break;
        case VR_F4_INDEX: x->f_index[3] = v[k]; break; case VR_F4_ASSET: x->f_asset[3] = v[k]; break;
        case VR_RELEASE_MODE: x->release_mode = v[k]; break;
        case VR_PREFERRED: x->preferred_robot = v[k]; break;
        case VR_FAIL_ASSET: x->fail_asset = v[k]; break;
        case VR_DEP1: x->dep1 = v[k]; break; case VR_DEP2: x->dep2 = v[k]; break;
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
static int selected_robot_active(Component *x, int robot) { for (int t = 0; t < MAXT; ++t) if (x->task_done[t] < 0.5 && (int)x->robot[t] == robot) return 1; return 0; }
static int uses_tool_active(Component *x) { for (int t = 0; t < MAXT; ++t) if (x->task_done[t] < 0.5 && x->tool[t] > 0.5) return 1; return 0; }
static void recompute_dep_mask(Component *x) {
    int mask = 0;
    for (int t = 0; t < MAXT; ++t) {
        if (x->task_done[t] > 0.5) continue;
        if ((int)x->robot[t] == 1) mask |= (int)x->dep1;
#ifdef MUTANT_OR_DEP_BIND
        if ((int)x->robot[t] == 2) mask |= (int)x->dep1;
#else
        if ((int)x->robot[t] == 2) mask |= (int)x->dep2;
#endif
        if (x->tool[t] > 0.5) mask |= (1 << 7);
    }
    x->dep_mask = (fmi3Float64)mask;
}
static void process_task_releases(Component *x) {
    if (x->release_mode < 0.5) return;
    int changed = 0;
    for (int t = 0; t < MAXT; ++t) {
        if (x->task_done[t] < 0.5 && x->task_count > t && x->end[t] <= x->time + 1e-9) {
            x->task_done[t] = 1; changed = 1;
            x->journal_failure_id = -1;
            int rb, hb, db, fb, pb, sb, tb;
            event_states(x, &rb, &hb, &db, &fb, &pb, &sb, &tb);
            char tid[16]; snprintf(tid, sizeof(tid), "%c", 'A' + t);
            record_transition(x, "TaskCompleted", tid, rb, hb, db, fb, pb, sb, tb);
        }
    }
    if (!changed) return;
    int needed[3]; unsigned long long needed_res = 0;
    for (int t = 0; t < MAXT; ++t) if (x->task_done[t] < 0.5 && x->task_count > t) needed_res |= (1ULL << ((int)x->robot[t] - 1));
    for (int t = 0; t < MAXT; ++t) {
        if (x->task_done[t] < 0.5) continue;
        int robot = (int)x->robot[t];
#ifdef MUTANT_PREMATURE_RELEASE
        if (robot > 0) {
#else
        if (robot > 0 && (needed_res & (1ULL << (robot - 1))) == 0) {
#endif
            int id = robot - 1;
            if (!((int)x->resource_released[0] & (1 << id))) {
                x->resource_released[0] += (fmi3Float64)(1 << id);
                x->journal_failure_id = -1;
                int rb, hb, db, fb, pb, sb, tb; event_states(x, &rb, &hb, &db, &fb, &pb, &sb, &tb);
                record_transition(x, "ResourceReleased", asset_name(id), rb, hb, db, fb, pb, sb, tb);
            }
        }
        if (x->tool[t] > 0.5 && !uses_tool_active(x) && !((int)x->resource_released[0] & (1 << 7))) {
            x->resource_released[0] += 128.0; x->journal_failure_id = -1;
            int rb, hb, db, fb, pb, sb, tb; event_states(x, &rb, &hb, &db, &fb, &pb, &sb, &tb);
            record_transition(x, "ResourceReleased", "tool-t1", rb, hb, db, fb, pb, sb, tb);
        }
    }
    int old_dep = (int)x->dep_mask;
    recompute_dep_mask(x);
#ifdef MUTANT_SHARED_DEP_LOSS
    if (x->task_done[0] > 0.5 && x->task_done[1] < 0.5 && (((int)x->dep_mask) & (1 << 3))) {
        x->dep_mask = (fmi3Float64)(((int)x->dep_mask) & ~(1 << 3));
    }
#endif
    int new_dep = (int)x->dep_mask;
    for (int bit = 0; bit < 8; ++bit) {
        int was = (old_dep >> bit) & 1, now = (new_dep >> bit) & 1;
        if (was && !now) {
            x->journal_dep_before = old_dep; x->journal_dep_after = new_dep; x->journal_failure_id = -1;
            int rb, hb, db, fb, pb, sb, tb; event_states(x, &rb, &hb, &db, &fb, &pb, &sb, &tb);
            record_transition(x, "DependencyDeactivated", asset_name(bit), rb, hb, db, fb, pb, sb, tb);
        } else if (!was && now) {
            x->journal_dep_before = old_dep; x->journal_dep_after = new_dep; x->journal_failure_id = -1;
            int rb, hb, db, fb, pb, sb, tb; event_states(x, &rb, &hb, &db, &fb, &pb, &sb, &tb);
            record_transition(x, "DependencyActivated", asset_name(bit), rb, hb, db, fb, pb, sb, tb);
        }
    }
}
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
        case VR_TOOL_ACQUIRED: v[k] = x->tool_acquired; break;
        case VR_TOOL_RELEASED: v[k] = x->tool_released; break;
        case VR_TOOL_MASK: v[k] = x->tool_mask; break;
        case VR_DEP_MASK: v[k] = x->dep_mask; break;
        case VR_FAIL_TARGET: v[k] = x->fail_target; break;
        case VR_DEP_PATH: v[k] = x->dep_path; break;
        case VR_AFFECTED: v[k] = x->affected; break;
        case 101: v[k] = x->journal_count; break;
        case 102: v[k] = x->journal_overflow; break;
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
        for (int fi = 0; fi < (int)x->fail_count && fi < 4; ++fi) {
            if (!(x->f_applied[fi] > 0.5) && x->f_time[fi] >= 0 && x->f_time[fi] > x->time + 1e-9 && x->f_time[fi] < boundary) boundary = x->f_time[fi];
        }
        if (x->completed < 0.5 && x->makespan > x->time + 1e-9 && x->makespan < boundary) boundary = x->makespan;
        if (x->release_mode > 0.5) {
            for (int tt = 0; tt < (int)x->task_count; ++tt) if (x->task_done[tt] < 0.5 && x->end[tt] > x->time + 1e-9 && x->end[tt] < boundary) boundary = x->end[tt];
        }
        fmi3Float64 piece = boundary - x->time;
        x->time += piece; remaining -= piece;
        int completion_due = x->completed < 0.5 && x->time >= x->makespan - 1e-9;
        /* Horizon boundary wins over exogenous events at the same timestamp;
           canonical processes completion before the horizon check. */
        if (x->time >= x->horizon - 1e-9) {
            if (completion_due) {
                process_task_releases(x);
                x->last_event = 2;
                apply_completion(x);
            }
            record_termination(x);
            x->last_event = 5;
            if (term) *term = true; if (last) *last = x->horizon;
            return fmi3OK;
        }
        /* Completion at makespan is processed before a same-timestamp failure. */
        if (completion_due) {
            process_task_releases(x);
            x->last_event = 2;
            apply_completion(x);
        }
        process_task_releases(x);
        {
            int due[4]; int due_count = 0; int hit = 0; int hit_failure = -1;
            for (int fi = 0; fi < (int)x->fail_count && fi < 4; ++fi) {
                if (!(x->f_applied[fi] > 0.5) && x->f_time[fi] >= 0 && x->time >= x->f_time[fi] - 1e-9) due[due_count++] = fi;
            }
            if (due_count > 0) {
                for (int d = 0; d < due_count; ++d) {
                    int fi = due[d];
                    x->journal_failure_id = fi;
                    x->failure_target_id = -1; resolve_failure_target_id(x, fi);
                    int rr, hh, dd, ff, pp, ss, tt;
                    event_states(x, &rr, &hh, &dd, &ff, &pp, &ss, &tt);
                    x->journal_before_mask = x->failed_mask;
                    if (x->failure_target_id >= 0) x->failed_mask |= (1ULL << (int)x->failure_target_id);
                    x->journal_after_mask = x->failed_mask;
                    x->fail_applied = x->f_time[fi]; x->f_applied[fi] = 1; x->last_event = 4;
                    record_transition(x, "AssetFailure", x->failure_target_name, rr, hh, dd, ff, pp, ss, tt);
                    if (x->f_time[fi] < x->makespan - 1e-9) {
                        int kind = (int)x->f_kind[fi];
                        if (kind == 1) { if (selected_robot_active(x, (int)x->f_index[fi])) { hit = 1; hit_failure = fi; x->fail_target = x->f_index[fi]; } }
                        else if (kind == 2) { x->tool_available = 0; x->tool_fail_applied = x->f_time[fi]; if (uses_tool_active(x)) { hit = 1; hit_failure = fi; x->fail_target = 3; } }
                        else if (kind == 3) { int asset=(int)x->f_asset[fi]; x->fail_target=asset; if (asset>=0 && (((int)x->dep_mask)&(1<<asset))) { hit=1; hit_failure=fi; x->dep_path=asset; } }
                    }
                }
                if (hit && x->f_time[hit_failure] < x->makespan - 1e-9) {
                    int rb, hb2, db2, fb2, pb2, sb2, tb2;
                    event_states(x, &rb, &hb2, &db2, &fb2, &pb2, &sb2, &tb2);
                    x->interrupted = 1; x->completed = 0;
                    for (int i = 0; i < MAXT; ++i) x->restored[i] = 0;
                    x->release = x->f_time[hit_failure];
                    if (x->tool_mask > 0) x->tool_released = x->f_time[hit_failure];
                    x->journal_failure_id = hit_failure;
                    record_transition(x, "RepairInterrupted", "a-lathe", rb, hb2, db2, fb2, pb2, sb2, tb2);
                    record_termination(x);
                    if (term) *term = true; if (last) *last = x->f_time[hit_failure];
                    return fmi3OK;
                }
            }
        }
        if (x->time >= x->horizon - 1e-9) {
            x->last_event = 5;
            record_termination(x);
            if (term) *term = true; if (last) *last = x->horizon;
            return fmi3OK;
        }
    }
    return fmi3OK;
}
fmi3Status fmi3GetString(fmi3Instance i, const fmi3ValueReference vr[], size_t n, const char *v[], size_t m) {
    Component *x = (Component *)i;
    if (!x || n != m) return fmi3Error;
    for (size_t k = 0; k < n; ++k) {
        if (vr[k] == 100) {
            snprintf(x->journal_output, sizeof(x->journal_output),
                     "{\"schema\":\"poie.s5v-n11-native-events/1\",\"capacity\":%d,\"count\":%d,\"overflow\":%s,\"events\":[%s]}",
                     JOURNAL_CAPACITY, (int)x->journal_count, x->journal_overflow > 0.5 ? "true" : "false", x->journal);
            v[k] = x->journal_output;
        } else {
            v[k] = "";
        }
    }
    return fmi3OK;
}
fmi3Status fmi3Terminate(fmi3Instance i) { (void)i; return fmi3OK; }
fmi3Status fmi3FreeInstance(fmi3Instance i) { free(i); return fmi3OK; }
fmi3Status fmi3Reset(fmi3Instance i) { (void)i; return fmi3OK; }
