/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Minimal FMI 3.0 Co-Simulation harness: unpack an FMU, dlopen its native
 * library, call the FMI lifecycle and advance communication steps.
 */
#include "fmi3.h"
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

typedef const char *(*get_version_t)(void);
typedef fmi3Instance (*instantiate_t)(const char *, const char *, const char *, fmi3Boolean, fmi3Boolean, fmi3Boolean,
                                      fmi3Boolean, const char *const *, size_t, fmi3InstanceEnvironment,
                                      fmi3LogMessageCallback, fmi3IntermediateUpdateCallback);
typedef fmi3Status (*enter_init_t)(fmi3Instance, fmi3Boolean, fmi3Float64, fmi3Float64, fmi3Boolean, fmi3Float64);
typedef fmi3Status (*exit_init_t)(fmi3Instance);
typedef fmi3Status (*do_step_t)(fmi3Instance, fmi3Float64, fmi3Float64, fmi3Boolean, fmi3Boolean *, fmi3Boolean *,
                                fmi3Boolean *, fmi3Float64 *);
typedef fmi3Status (*set_f64_t)(fmi3Instance, const fmi3ValueReference[], size_t, const fmi3Float64[], size_t);
typedef fmi3Status (*get_f64_t)(fmi3Instance, const fmi3ValueReference[], size_t, fmi3Float64[], size_t);
typedef fmi3Status (*terminate_t)(fmi3Instance);
typedef fmi3Status (*free_instance_t)(fmi3Instance);

static int csv_snapshot(FILE *out, fmi3Instance instance, get_f64_t get_f64, fmi3Boolean terminated) {
    fmi3ValueReference vr[] = {1, 2, 10, 11, 12, 13, 14};
    fmi3Float64 value[7];
    if (get_f64(instance, vr, 7, value, 7) != fmi3OK) return 1;
    fprintf(out, "%.9f,%.9f,%.9f,%.9f,%.9f,%.9f,%.9f,%d\n", value[0], value[1], value[2], value[3], value[4],
            value[5], value[6], terminated ? 1 : 0);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 4) {
        fprintf(stderr, "usage: %s <fmu-path> <step-seconds> <output-csv>\n", argv[0]);
        return 2;
    }
    const char *fmu = argv[1];
    double step = atof(argv[2]);
    const char *csv_path = argv[3];
    char unpacked[] = "/tmp/poie-s5v-fmu-XXXXXX";
    char command[4096];
    if (mkdtemp(unpacked) == NULL) { perror("mkdtemp"); return 2; }
    snprintf(command, sizeof(command), "unzip -q '%s' -d '%s'", fmu, unpacked);
    if (system(command) != 0) { fprintf(stderr, "unzip failed\n"); return 2; }
    char library[4096];
#if defined(__APPLE__)
    const char *platform = "Darwin";
    const char *library_name = "model.dylib";
#elif defined(__linux__)
    const char *platform = "Linux";
    const char *library_name = "model.so";
#else
#error "unsupported platform"
#endif
#if defined(__aarch64__) || defined(__arm64__)
    const char *architecture = "aarch64";
#elif defined(__x86_64__)
    const char *architecture = "x86_64";
#else
#error "unsupported architecture"
#endif
    snprintf(library, sizeof(library), "%s/binaries/%s/%s/%s", unpacked, architecture, platform, library_name);
    void *handle = dlopen(library, RTLD_NOW | RTLD_LOCAL);
    if (handle == NULL) { fprintf(stderr, "dlopen: %s\n", dlerror()); return 2; }
    get_version_t get_version = (get_version_t)dlsym(handle, "fmi3GetVersion");
    instantiate_t instantiate = (instantiate_t)dlsym(handle, "fmi3InstantiateCoSimulation");
    enter_init_t enter_init = (enter_init_t)dlsym(handle, "fmi3EnterInitializationMode");
    exit_init_t exit_init = (exit_init_t)dlsym(handle, "fmi3ExitInitializationMode");
    do_step_t do_step = (do_step_t)dlsym(handle, "fmi3DoStep");
    set_f64_t set_f64 = (set_f64_t)dlsym(handle, "fmi3SetFloat64");
    get_f64_t get_f64 = (get_f64_t)dlsym(handle, "fmi3GetFloat64");
    terminate_t terminate = (terminate_t)dlsym(handle, "fmi3Terminate");
    free_instance_t free_instance = (free_instance_t)dlsym(handle, "fmi3FreeInstance");
    if (!get_version || !instantiate || !enter_init || !exit_init || !do_step || !set_f64 || !get_f64 ||
        !terminate || !free_instance) {
        fprintf(stderr, "missing FMI symbol\n");
        return 2;
    }
    if (strcmp(get_version(), "3.0") != 0) { fprintf(stderr, "not FMI 3.0\n"); return 2; }
    fmi3Instance instance = instantiate("harness", "{9a4f79b0-5d4b-4f4e-a6b1-s5vchallenge01}", unpacked, false, false,
                                        false, false, NULL, 0, NULL, NULL, NULL);
    if (instance == NULL) { fprintf(stderr, "instantiate failed\n"); return 2; }
    fmi3ValueReference params[] = {3, 4, 5, 6, 7, 8, 9};
    fmi3Float64 values[] = {2500.0, 3000.0, 720000000.0, 10000.0, 144000000.0, 1800000.0, 10800.0};
    if (set_f64(instance, params, 7, values, 7) != fmi3OK) { fprintf(stderr, "set parameters failed\n"); return 2; }
    if (enter_init(instance, false, 0.0, 0.0, false, 0.0) != fmi3OK) return 2;
    if (exit_init(instance) != fmi3OK) return 2;
    FILE *out = fopen(csv_path, "w");
    if (out == NULL) { perror(csv_path); return 2; }
    fprintf(out, "time_s,energy_J,generated_J,load_J,repair_J,curtailed_J,depleted,terminated\n");
    if (csv_snapshot(out, instance, get_f64, false) != 0) return 2;
    fmi3Float64 current = 0.0;
    for (int i = 0; i < 400; ++i) {
        fmi3Boolean event_handled = false, terminated = false, early = false;
        fmi3Float64 last = current;
        if (do_step(instance, current, step, true, &event_handled, &terminated, &early, &last) != fmi3OK) {
            fprintf(stderr, "doStep failed at %.3f\n", current);
            return 2;
        }
        current = terminated ? last : current + step;
        if (csv_snapshot(out, instance, get_f64, terminated) != 0) return 2;
        if (terminated) break;
        if (current > 500.0 * 3600.0) break;
    }
    fclose(out);
    terminate(instance);
    free_instance(instance);
    dlclose(handle);
    char cleanup[4096];
    snprintf(cleanup, sizeof(cleanup), "rm -rf '%s'", unpacked);
    system(cleanup);
    return 0;
}
