/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
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

static int snap(FILE *out, fmi3Instance instance, get_f64_t get, fmi3Boolean terminated) {
    fmi3ValueReference vr[] = {1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22};
    fmi3Float64 v[14];
    if (get(instance, vr, 14, v, 14) != fmi3OK) return 1;
    for (int i = 0; i < 14; ++i) fprintf(out, "%.9f,", v[i]);
    fprintf(out, "%d\n", terminated ? 1 : 0);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 9) {
        fprintf(stderr, "usage: %s <fmu> <step> <task-mode> <robots> <failure-time> <failure-robot> <stop> <out>\n", argv[0]);
        return 2;
    }
    const char *fmu = argv[1];
    double step = atof(argv[2]);
    double mode = atof(argv[3]);
    double robots = atof(argv[4]);
    double failure = atof(argv[5]);
    double failure_robot = atof(argv[6]);
    double stop = atof(argv[7]);
    const char *csv = argv[8];
    char dir[] = "/tmp/poie-n5-XXXXXX";
    if (mkdtemp(dir) == NULL) return 2;
    char cmd[4096];
    snprintf(cmd, sizeof(cmd), "unzip -q '%s' -d '%s'", fmu, dir);
    if (system(cmd) != 0) return 2;
#if defined(__APPLE__)
    const char *platform = "darwin", *libname = "poie_n5_scheduler.dylib";
#else
    const char *platform = "linux", *libname = "poie_n5_scheduler.so";
#endif
#if defined(__aarch64__) || defined(__arm64__)
    const char *arch = "aarch64";
#else
    const char *arch = "x86_64";
#endif
    char lib[4096];
    snprintf(lib, sizeof(lib), "%s/binaries/%s-%s/%s", dir, arch, platform, libname);
    void *h = dlopen(lib, RTLD_NOW | RTLD_LOCAL);
    if (!h) { fprintf(stderr, "%s\n", dlerror()); return 2; }
    get_version_t gv = (get_version_t)dlsym(h, "fmi3GetVersion");
    instantiate_t inst = (instantiate_t)dlsym(h, "fmi3InstantiateCoSimulation");
    enter_init_t enter = (enter_init_t)dlsym(h, "fmi3EnterInitializationMode");
    exit_init_t exit = (exit_init_t)dlsym(h, "fmi3ExitInitializationMode");
    do_step_t ds = (do_step_t)dlsym(h, "fmi3DoStep");
    set_f64_t set = (set_f64_t)dlsym(h, "fmi3SetFloat64");
    get_f64_t get = (get_f64_t)dlsym(h, "fmi3GetFloat64");
    terminate_t term = (terminate_t)dlsym(h, "fmi3Terminate");
    free_instance_t freei = (free_instance_t)dlsym(h, "fmi3FreeInstance");
    if (!gv || !inst || !enter || !exit || !ds || !set || !get || !term || !freei) return 2;
    if (strcmp(gv(), "3.0") != 0) return 2;
    fmi3Instance i = inst("harness", "{d8e3c0a4-5b6f-4c1d-8e4a-n5scheduler001}", dir, false, false, false, false,
                          NULL, 0, NULL, NULL, NULL);
    if (!i) return 2;
    fmi3ValueReference pv[] = {2, 3, 4, 5, 6};
    fmi3Float64 p[] = {mode, robots, failure, failure_robot, 72000.0};
    if (set(i, pv, 5, p, 5) != fmi3OK) return 2;
    if (enter(i, false, 0.0, 0.0, false, 0.0) != fmi3OK) return 2;
    if (exit(i) != fmi3OK) return 2;
    FILE *out = fopen(csv, "w");
    if (!out) return 2;
    fprintf(out, "time_s,makespan_s,completedCount,interrupted,failureAppliedTime_s,t1Start_s,t1End_s,t1Robot,"
                 "t2Start_s,t2End_s,t2Robot,pumpRestored,latheRestored,lastEventCode,terminated\n");
    if (snap(out, i, get, false) != 0) return 2;
    double current = 0.0;
    for (int n = 0; n < 4000 && current < stop - 1e-9; ++n) {
        fmi3Boolean eh = false, term_sim = false, early = false;
        fmi3Float64 last = current;
        double step_now = current + step > stop ? stop - current : step;
        if (ds(i, current, step_now, true, &eh, &term_sim, &early, &last) != fmi3OK) return 2;
        current = term_sim ? last : current + step_now;
        if (snap(out, i, get, term_sim) != 0) return 2;
        if (term_sim) break;
    }
    fclose(out);
    term(i); freei(i); dlclose(h);
    snprintf(cmd, sizeof(cmd), "rm -rf '%s'", dir); system(cmd);
    return 0;
}
