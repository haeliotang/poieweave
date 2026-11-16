/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Independent FMI 3.0 Co-Simulation model: PV + battery + load + initial
 * repair draw. It does not call or link the Poieweave Rust engine.
 */
#include "fmi3.h"
#include <math.h>
#include <stdlib.h>
#include <string.h>

#define VR_TIME 1
#define VR_ENERGY 2
#define VR_PV_POWER 3
#define VR_LOAD_POWER 4
#define VR_CAPACITY 5
#define VR_MAX_CHARGE 6
#define VR_INITIAL_ENERGY 7
#define VR_REPAIR_ENERGY 8
#define VR_REPAIR_DURATION 9
#define VR_GENERATED 10
#define VR_LOAD_ENERGY 11
#define VR_REPAIR_CONSUMED 12
#define VR_CURTAILED 13
#define VR_DEPLETED 14
#define VR_PV_FAILURE_TIME 15
#define VR_PV_AVAILABLE 16
#define VR_FAILURE_APPLIED_TIME 17
#define VR_REPAIR_COMPLETED_TIME 18
#define VR_LAST_EVENT_CODE 19
#define VR_LAST_EVENT_TIME 20

typedef struct {
    fmi3Float64 time;
    fmi3Float64 energy;
    fmi3Float64 pv_power;
    fmi3Float64 load_power;
    fmi3Float64 capacity;
    fmi3Float64 max_charge;
    fmi3Float64 initial_energy;
    fmi3Float64 repair_energy;
    fmi3Float64 repair_duration;
    fmi3Float64 generated;
    fmi3Float64 load_energy;
    fmi3Float64 repair_consumed;
    fmi3Float64 curtailed;
    fmi3Float64 depleted;
    fmi3Float64 pv_failure_time;
    fmi3Float64 failure_applied_time;
    fmi3Float64 pv_failed;
    fmi3Float64 repair_completed_time;
    fmi3Float64 last_event_code;
    fmi3Float64 last_event_time;
} Component;

const char *fmi3GetVersion(void) { return "3.0"; }

fmi3Status fmi3SetDebugLogging(fmi3Instance instance, fmi3Boolean logging_on, size_t n_categories,
                               const char *categories[]) {
    (void)instance;
    (void)logging_on;
    (void)n_categories;
    (void)categories;
    return fmi3OK;
}

fmi3Instance fmi3InstantiateCoSimulation(const char *instance_name, const char *instantiation_token,
                                         const char *resource_path, fmi3Boolean visible, fmi3Boolean logging_on,
                                         fmi3Boolean event_mode_used, fmi3Boolean early_return_allowed,
                                         const char *const *required_intermediate_variables,
                                         size_t n_required_intermediate_variables,
                                         fmi3InstanceEnvironment instance_environment,
                                         fmi3LogMessageCallback log_message,
                                         fmi3IntermediateUpdateCallback intermediate_update) {
    (void)instance_name;
    (void)instantiation_token;
    (void)resource_path;
    (void)visible;
    (void)logging_on;
    (void)event_mode_used;
    (void)early_return_allowed;
    (void)required_intermediate_variables;
    (void)n_required_intermediate_variables;
    (void)instance_environment;
    (void)log_message;
    (void)intermediate_update;
    Component *component = (Component *)calloc(1, sizeof(Component));
    if (component == NULL) return NULL;
    component->pv_power = 2500.0;
    component->load_power = 3000.0;
    component->capacity = 200.0 * 3600000.0;
    component->max_charge = 10000.0;
    component->initial_energy = 40.0 * 3600000.0;
    component->repair_energy = 0.5 * 3600000.0;
    component->repair_duration = 3.0 * 3600.0;
    component->pv_failure_time = -1.0;
    component->failure_applied_time = -1.0;
    component->pv_failed = 0.0;
    component->repair_completed_time = -1.0;
    component->last_event_code = 0.0;
    component->last_event_time = -1.0;
    return component;
}

fmi3Status fmi3EnterInitializationMode(fmi3Instance instance, fmi3Boolean tolerance_defined,
                                       fmi3Float64 tolerance, fmi3Float64 start_time,
                                       fmi3Boolean stop_time_defined, fmi3Float64 stop_time) {
    (void)tolerance_defined;
    (void)tolerance;
    (void)stop_time_defined;
    (void)stop_time;
    Component *c = (Component *)instance;
    if (c == NULL) return fmi3Error;
    c->time = start_time;
    c->energy = c->initial_energy;
    c->generated = 0.0;
    c->load_energy = 0.0;
    c->repair_consumed = 0.0;
    c->curtailed = 0.0;
    c->depleted = 0.0;
    c->pv_failed = 0.0;
    c->failure_applied_time = -1.0;
    c->repair_completed_time = -1.0;
    c->last_event_code = 0.0;
    c->last_event_time = -1.0;
    return fmi3OK;
}

fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) { return instance == NULL ? fmi3Error : fmi3OK; }

fmi3Status fmi3SetFloat64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          const fmi3Float64 value[], size_t n_values) {
    Component *c = (Component *)instance;
    if (c == NULL || n_values != nvr) return fmi3Error;
    for (size_t i = 0; i < nvr; ++i) {
        switch (vr[i]) {
        case VR_PV_POWER: c->pv_power = value[i]; break;
        case VR_LOAD_POWER: c->load_power = value[i]; break;
        case VR_CAPACITY: c->capacity = value[i]; break;
        case VR_MAX_CHARGE: c->max_charge = value[i]; break;
        case VR_INITIAL_ENERGY: c->initial_energy = value[i]; c->energy = value[i]; break;
        case VR_REPAIR_ENERGY: c->repair_energy = value[i]; break;
        case VR_REPAIR_DURATION: c->repair_duration = value[i]; break;
        case VR_PV_FAILURE_TIME: c->pv_failure_time = value[i]; break;
        default: return fmi3Error;
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
        case VR_ENERGY: value[i] = c->energy; break;
        case VR_PV_POWER: value[i] = c->pv_power; break;
        case VR_LOAD_POWER: value[i] = c->load_power; break;
        case VR_CAPACITY: value[i] = c->capacity; break;
        case VR_GENERATED: value[i] = c->generated; break;
        case VR_LOAD_ENERGY: value[i] = c->load_energy; break;
        case VR_REPAIR_CONSUMED: value[i] = c->repair_consumed; break;
        case VR_CURTAILED: value[i] = c->curtailed; break;
        case VR_DEPLETED: value[i] = c->depleted; break;
        case VR_PV_AVAILABLE: value[i] = c->pv_failed > 0.5 ? 0.0 : 1.0; break;
        case VR_FAILURE_APPLIED_TIME: value[i] = c->failure_applied_time; break;
        case VR_REPAIR_COMPLETED_TIME: value[i] = c->repair_completed_time; break;
        case VR_LAST_EVENT_CODE: value[i] = c->last_event_code; break;
        case VR_LAST_EVENT_TIME: value[i] = c->last_event_time; break;
        default: return fmi3Error;
        }
    }
    return fmi3OK;
}

static fmi3Float64 repair_rate(const Component *c) {
    if (c->repair_duration <= 0.0 || c->repair_energy <= 0.0) return 0.0;
    if (c->time >= c->repair_duration) return 0.0;
    if (c->repair_consumed >= c->repair_energy) return 0.0;
    return c->repair_energy / c->repair_duration;
}

fmi3Status fmi3DoStep(fmi3Instance instance, fmi3Float64 current_communication_point,
                      fmi3Float64 communication_step_size,
                      fmi3Boolean no_set_fmu_state_prior_to_current_point, fmi3Boolean *event_handled,
                      fmi3Boolean *terminate_simulation, fmi3Boolean *early_return,
                      fmi3Float64 *last_successful_time) {
    (void)no_set_fmu_state_prior_to_current_point;
    Component *c = (Component *)instance;
    if (c == NULL) return fmi3Error;
    if (event_handled) *event_handled = true;
    if (terminate_simulation) *terminate_simulation = false;
    if (early_return) *early_return = false;
    if (last_successful_time) *last_successful_time = c->time;
    if (fabs(current_communication_point - c->time) > 1e-9) return fmi3Error;
    fmi3Float64 remaining = communication_step_size;
    if (!c->pv_failed && c->pv_failure_time >= 0.0 && c->time >= c->pv_failure_time - 1e-9) {
        c->pv_failed = 1.0;
        c->failure_applied_time = c->pv_failure_time;
    }
    while (remaining > 1e-12) {
        fmi3Float64 rate = repair_rate(c);
        fmi3Float64 pv = c->pv_failed ? 0.0 : fmin(c->pv_power, c->max_charge);
        fmi3Float64 net = c->load_power + rate - pv;
        fmi3Float64 piece = remaining;
        if (rate > 0.0) {
            fmi3Float64 to_repair_end = c->repair_duration - c->time;
            if (to_repair_end > 0.0 && to_repair_end < piece) piece = to_repair_end;
        }
        if (!c->pv_failed && c->pv_failure_time > c->time + 1e-9) {
            fmi3Float64 to_failure = c->pv_failure_time - c->time;
            if (to_failure < piece) piece = to_failure;
        }
        if (net > 0.0 && c->energy <= net * piece + 1e-12) {
            fmi3Float64 t_empty = c->energy / net;
            if (t_empty < 0.0) t_empty = 0.0;
            c->time += t_empty;
            c->generated += pv * t_empty;
            c->load_energy += c->load_power * t_empty;
            c->repair_consumed += rate * t_empty;
            c->energy = 0.0;
            c->depleted = 1.0;
            if (terminate_simulation) *terminate_simulation = true;
            if (last_successful_time) *last_successful_time = c->time;
            break;
        }
        c->energy -= net * piece;
        c->generated += pv * piece;
        c->load_energy += c->load_power * piece;
        c->repair_consumed += rate * piece;
        c->time += piece;
        remaining -= piece;
        if (c->repair_completed_time < 0.0 && c->repair_duration > 0.0 && c->time >= c->repair_duration - 1e-9) {
            c->repair_completed_time = c->repair_duration;
            c->last_event_code = 2.0;
            c->last_event_time = c->repair_duration;
        }
        if (!c->pv_failed && c->pv_failure_time >= 0.0 && c->time >= c->pv_failure_time - 1e-9) {
            c->pv_failed = 1.0;
            c->failure_applied_time = c->pv_failure_time;
            c->last_event_code = 1.0;
            c->last_event_time = c->pv_failure_time;
        }
        if (c->energy > c->capacity) {
            c->curtailed += c->energy - c->capacity;
            c->energy = c->capacity;
        }
    }
    return fmi3OK;
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    (void)instance;
    return fmi3OK;
}

fmi3Status fmi3FreeInstance(fmi3Instance instance) {
    free(instance);
    return fmi3OK;
}

fmi3Status fmi3Reset(fmi3Instance instance) {
    Component *c = (Component *)instance;
    if (c == NULL) return fmi3Error;
    c->time = 0.0;
    c->energy = c->initial_energy;
    c->generated = 0.0;
    c->load_energy = 0.0;
    c->repair_consumed = 0.0;
    c->curtailed = 0.0;
    c->depleted = 0.0;
    return fmi3OK;
}
