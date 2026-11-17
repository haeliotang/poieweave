/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Independent deterministic scheduler: two fixed repair tasks, one robot per
 * task from an explicit pool. No Poieweave scheduler/engine linkage.
 */
#include "fmi3.h"
#include <math.h>
#include <stdlib.h>
#include <string.h>

#define VR_TIME 1
#define VR_TASK_MODE 2
#define VR_ROBOT_COUNT 3
#define VR_FAILURE_TIME 4
#define VR_FAILURE_ROBOT 5
#define VR_HORIZON 6
#define VR_MAKESPAN 10
#define VR_COMPLETED 11
#define VR_INTERRUPTED 12
#define VR_FAILURE_APPLIED 13
#define VR_T1_START 14
#define VR_T1_END 15
#define VR_T1_ROBOT 16
#define VR_T2_START 17
#define VR_T2_END 18
#define VR_T2_ROBOT 19
#define VR_PUMP_RESTORED 20
#define VR_LATHE_RESTORED 21
#define VR_LAST_EVENT 22

typedef struct {
    fmi3Float64 time;
    fmi3Float64 task_mode;
    fmi3Float64 robot_count;
    fmi3Float64 failure_time;
    fmi3Float64 failure_robot;
    fmi3Float64 horizon;
    fmi3Float64 makespan;
    fmi3Float64 completed;
    fmi3Float64 interrupted;
    fmi3Float64 failure_applied_time;
    fmi3Float64 t1_start, t1_end, t1_robot;
    fmi3Float64 t2_start, t2_end, t2_robot;
    fmi3Float64 pump_restored, lathe_restored;
    fmi3Float64 last_event;
} Component;

const char *fmi3GetVersion(void) { return "3.0"; }
fmi3Status fmi3SetDebugLogging(fmi3Instance instance, fmi3Boolean logging_on, size_t n,
                               const char *categories[]) {
    (void)instance; (void)logging_on; (void)n; (void)categories; return fmi3OK;
}

static void schedule(Component *c) {
    fmi3Float64 available[2] = {0.0, 0.0};
    int robots = (int)c->robot_count;
    if (robots < 1) robots = 1;
    if (robots > 2) robots = 2;
    /* priority: lathe first when two tasks, then pump */
    if (c->task_mode > 0.5) {
        c->t1_start = 0.0; c->t1_end = 7200.0; c->t1_robot = 1.0;
        c->t2_start = c->t1_end;
        c->t2_end = c->t2_start + 10800.0;
        c->t2_robot = 1.0;
        if (robots >= 2) { c->t2_start = 0.0; c->t2_end = 10800.0; c->t2_robot = 2.0; }
        c->makespan = fmax(c->t1_end, c->t2_end);
    } else {
        c->t1_start = 0.0; c->t1_end = 10800.0; c->t1_robot = 1.0;
        c->t2_start = -1.0; c->t2_end = -1.0; c->t2_robot = 0.0;
        c->makespan = 10800.0;
    }
}

fmi3Instance fmi3InstantiateCoSimulation(const char *instance_name, const char *instantiation_token,
                                         const char *resource_path, fmi3Boolean visible, fmi3Boolean logging_on,
                                         fmi3Boolean event_mode_used, fmi3Boolean early_return_allowed,
                                         const char *const *required_intermediate_variables,
                                         size_t n_required_intermediate_variables,
                                         fmi3InstanceEnvironment instance_environment,
                                         fmi3LogMessageCallback log_message,
                                         fmi3IntermediateUpdateCallback intermediate_update) {
    (void)instance_name; (void)instantiation_token; (void)resource_path; (void)visible; (void)logging_on;
    (void)event_mode_used; (void)early_return_allowed; (void)required_intermediate_variables;
    (void)n_required_intermediate_variables; (void)instance_environment; (void)log_message; (void)intermediate_update;
    Component *c = (Component *)calloc(1, sizeof(Component));
    if (c == NULL) return NULL;
    c->task_mode = 0.0; c->robot_count = 1.0; c->failure_time = -1.0; c->failure_robot = 0.0;
    c->horizon = 72000.0;
    return c;
}

fmi3Status fmi3EnterInitializationMode(fmi3Instance instance, fmi3Boolean td, fmi3Float64 tol,
                                       fmi3Float64 start_time, fmi3Boolean st, fmi3Float64 stop_time) {
    (void)td; (void)tol; (void)st; (void)stop_time;
    Component *c = (Component *)instance;
    if (c == NULL) return fmi3Error;
    c->time = start_time; c->completed = 0.0; c->interrupted = 0.0;
    c->failure_applied_time = -1.0; c->pump_restored = 0.0; c->lathe_restored = 0.0; c->last_event = 0.0;
    schedule(c);
    return fmi3OK;
}
fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) { return instance == NULL ? fmi3Error : fmi3OK; }

fmi3Status fmi3SetFloat64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          const fmi3Float64 value[], size_t n_values) {
    Component *c = (Component *)instance;
    if (c == NULL || n_values != nvr) return fmi3Error;
    for (size_t i = 0; i < nvr; ++i) {
        switch (vr[i]) {
        case VR_TASK_MODE: c->task_mode = value[i]; break;
        case VR_ROBOT_COUNT: c->robot_count = value[i]; break;
        case VR_FAILURE_TIME: c->failure_time = value[i]; break;
        case VR_FAILURE_ROBOT: c->failure_robot = value[i]; break;
        case VR_HORIZON: c->horizon = value[i]; break;
        default: break;
        }
    }
    return fmi3OK;
}

fmi3Status fmi3GetFloat64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          fmi3Float64 value[], size_t n_values) {
    Component *c = (Component *)instance;
    if (c == NULL || n_values != nvr) return fmi3Error;
    for (size_t i = 0; i < nvr; ++i) {
        switch (vr[i]) {
        case VR_TIME: value[i] = c->time; break;
        case VR_MAKESPAN: value[i] = c->makespan; break;
        case VR_COMPLETED: value[i] = c->completed; break;
        case VR_INTERRUPTED: value[i] = c->interrupted; break;
        case VR_FAILURE_APPLIED: value[i] = c->failure_applied_time; break;
        case VR_T1_START: value[i] = c->t1_start; break;
        case VR_T1_END: value[i] = c->t1_end; break;
        case VR_T1_ROBOT: value[i] = c->t1_robot; break;
        case VR_T2_START: value[i] = c->t2_start; break;
        case VR_T2_END: value[i] = c->t2_end; break;
        case VR_T2_ROBOT: value[i] = c->t2_robot; break;
        case VR_PUMP_RESTORED: value[i] = c->pump_restored; break;
        case VR_LATHE_RESTORED: value[i] = c->lathe_restored; break;
        case VR_LAST_EVENT: value[i] = c->last_event; break;
        default: value[i] = 0.0; break;
        }
    }
    return fmi3OK;
}

static int selected(Component *c, int robot) {
    if (c->task_mode > 0.5) {
        return ((int)c->t1_robot == robot || (int)c->t2_robot == robot);
    }
    return (int)c->t1_robot == robot;
}

fmi3Status fmi3DoStep(fmi3Instance instance, fmi3Float64 current, fmi3Float64 step,
                      fmi3Boolean noset, fmi3Boolean *event_handled, fmi3Boolean *terminate_simulation,
                      fmi3Boolean *early_return, fmi3Float64 *last_successful_time) {
    (void)noset;
    Component *c = (Component *)instance;
    if (c == NULL) return fmi3Error;
    if (event_handled) *event_handled = true;
    if (terminate_simulation) *terminate_simulation = false;
    if (early_return) *early_return = false;
    if (last_successful_time) *last_successful_time = c->time;
    if (fabs(current - c->time) > 1e-9) return fmi3Error;
    fmi3Float64 remaining = step;
    while (remaining > 1e-12) {
        fmi3Float64 target = c->time + remaining;
        fmi3Float64 boundary = fmin(target, c->horizon);
        if (c->failure_time >= 0.0 && c->failure_time > c->time + 1e-9 && c->failure_time < boundary) {
            boundary = c->failure_time;
        }
        if (c->completed < 0.5 && c->makespan > c->time + 1e-9 && c->makespan < boundary) {
            boundary = c->makespan;
        }
        fmi3Float64 piece = boundary - c->time;
        c->time += piece;
        remaining -= piece;
        if (c->failure_time >= 0.0 && c->failure_applied_time < 0.0 && c->time >= c->failure_time - 1e-9) {
            c->failure_applied_time = c->failure_time;
            c->last_event = 4.0;
            if (selected(c, (int)c->failure_robot) && c->failure_time < c->makespan - 1e-9) {
                c->interrupted = 1.0;
                c->completed = 0.0;
                c->pump_restored = 0.0;
                c->lathe_restored = 0.0;
                if (terminate_simulation) *terminate_simulation = true;
                if (last_successful_time) *last_successful_time = c->failure_time;
                return fmi3OK;
            }
        }
        if (c->completed < 0.5 && c->time >= c->makespan - 1e-9) {
            c->completed = (c->task_mode > 0.5) ? 2.0 : 1.0;
            c->last_event = 2.0;
            c->pump_restored = 1.0;
            if (c->task_mode > 0.5) c->lathe_restored = 1.0;
        }
        if (c->time >= c->horizon - 1e-9) {
            c->last_event = 5.0;
            if (terminate_simulation) *terminate_simulation = true;
            if (last_successful_time) *last_successful_time = c->horizon;
            return fmi3OK;
        }
    }
    return fmi3OK;
}

fmi3Status fmi3Terminate(fmi3Instance instance) { (void)instance; return fmi3OK; }
fmi3Status fmi3FreeInstance(fmi3Instance instance) { free(instance); return fmi3OK; }
fmi3Status fmi3Reset(fmi3Instance instance) { (void)instance; return fmi3OK; }
