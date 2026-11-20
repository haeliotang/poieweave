/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 */
#include "fmi3.h"
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

typedef const char *(*gv_t)(void);
typedef fmi3Instance (*inst_t)(const char *, const char *, const char *, fmi3Boolean, fmi3Boolean, fmi3Boolean,
                               fmi3Boolean, const char *const *, size_t, fmi3InstanceEnvironment,
                               fmi3LogMessageCallback, fmi3IntermediateUpdateCallback);
typedef fmi3Status (*init_t)(fmi3Instance, fmi3Boolean, fmi3Float64, fmi3Float64, fmi3Boolean, fmi3Float64);
typedef fmi3Status (*exit_t)(fmi3Instance);
typedef fmi3Status (*step_t)(fmi3Instance, fmi3Float64, fmi3Float64, fmi3Boolean, fmi3Boolean *, fmi3Boolean *,
                             fmi3Boolean *, fmi3Float64 *);
typedef fmi3Status (*set_t)(fmi3Instance, const fmi3ValueReference[], size_t, const fmi3Float64[], size_t);
typedef fmi3Status (*get_t)(fmi3Instance, const fmi3ValueReference[], size_t, fmi3Float64[], size_t);
typedef fmi3Status (*term_t)(fmi3Instance);
typedef fmi3Status (*free_t)(fmi3Instance);

static int read_contract(const char *path, fmi3ValueReference *vr, fmi3Float64 *val, size_t *count) {
    static const char *keys[] = {"taskCount","t1Duration_s","t1Priority","t1RobotMask","t1RequiresTool",
        "t2Duration_s","t2Priority","t2RobotMask","t2RequiresTool","t3Duration_s","t3Priority","t3RobotMask",
        "t3RequiresTool","robotCount","failureTime_s","failureKind","failureIndex","horizon_s",
        "preferredRobot","failureAssetId","robot1DependencyMask","robot2DependencyMask"};
    static const fmi3ValueReference refs[] = {2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,43,44,45,46};
    FILE *f = fopen(path, "r");
    if (!f) return 1;
    char line[512];
    *count = 0;
    while (fgets(line, sizeof(line), f)) {
        char *eq = strchr(line, '=');
        if (!eq) continue;
        *eq = 0;
        for (size_t i = 0; i < sizeof(keys) / sizeof(keys[0]); ++i) {
            if (strcmp(line, keys[i]) == 0) {
                vr[*count] = refs[i];
                val[*count] = atof(eq + 1);
                (*count)++;
                break;
            }
        }
    }
    fclose(f);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 5) { fprintf(stderr, "usage: %s <fmu> <step> <contract> <out>\n", argv[0]); return 2; }
    char dir[] = "/tmp/poie-n6-XXXXXX";
    if (mkdtemp(dir) == NULL) return 2;
    char cmd[4096];
    snprintf(cmd, sizeof(cmd), "unzip -q '%s' -d '%s'", argv[1], dir);
    if (system(cmd) != 0) return 2;
#if defined(__APPLE__)
    const char *platform = "darwin", *libname = "poie_n10_scheduler.dylib";
#else
    const char *platform = "linux", *libname = "poie_n10_scheduler.so";
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
    gv_t gv = (gv_t)dlsym(h, "fmi3GetVersion");
    inst_t inst = (inst_t)dlsym(h, "fmi3InstantiateCoSimulation");
    init_t enter = (init_t)dlsym(h, "fmi3EnterInitializationMode");
    exit_t exitm = (exit_t)dlsym(h, "fmi3ExitInitializationMode");
    step_t ds = (step_t)dlsym(h, "fmi3DoStep");
    set_t set = (set_t)dlsym(h, "fmi3SetFloat64");
    get_t get = (get_t)dlsym(h, "fmi3GetFloat64");
    term_t term = (term_t)dlsym(h, "fmi3Terminate");
    free_t freei = (free_t)dlsym(h, "fmi3FreeInstance");
    if (!gv || !inst || !enter || !exitm || !ds || !set || !get || !term || !freei) return 2;
    if (strcmp(gv(), "3.0") != 0) return 2;
    fmi3Instance i = inst("harness", "{e9f4d1b5-6c7a-4d2e-9f5b-n10scheduler01}", dir, false, false, false, false,
                          NULL, 0, NULL, NULL, NULL);
    if (!i) return 2;
    fmi3ValueReference vr[32]; fmi3Float64 val[32]; size_t n = 0;
    if (read_contract(argv[3], vr, val, &n) != 0 || n == 0) return 2;
    if (set(i, vr, n, val, n) != fmi3OK) return 2;
    double stop = 72000.0;
    for (size_t k = 0; k < n; ++k) if (vr[k] == 19) stop = val[k];
    if (enter(i, false, 0.0, 0.0, false, 0.0) != fmi3OK) return 2;
    if (exitm(i) != fmi3OK) return 2;
    FILE *out = fopen(argv[4], "w");
    if (!out) return 2;
    fprintf(out, "time_s,makespan_s,completedCount,interrupted,failureAppliedTime_s,t1Start_s,t1End_s,t1Robot,"
                 "t2Start_s,t2End_s,t2Robot,t3Start_s,t3End_s,t3Robot,restored1,restored2,restored3,lastEventCode,"
                 "toolAvailable,toolFailureAppliedTime_s,release_s,toolAcquired_s,toolReleased_s,toolTaskMask,"
                 "preferredRobot,failureAssetId,robot1DependencyMask,robot2DependencyMask,selectedDependencyMask,"
                 "failureTargetId,dependencyPathCode,affectedSelectedResource,terminated\n");
    fmi3ValueReference ov[] = {1,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50};
    fmi3Float64 ovv[32];
    get(i, ov, 32, ovv, 32);
    for (int k = 0; k < 32; ++k) fprintf(out, "%.9f,", ovv[k]);
    fprintf(out, "0\n");
    double current = 0.0;
    for (int s = 0; s < 4000 && current < stop - 1e-9; ++s) {
        fmi3Boolean eh = false, term_sim = false, early = false;
        fmi3Float64 last = current;
        double step_now = current + atof(argv[2]) > stop ? stop - current : atof(argv[2]);
        if (ds(i, current, step_now, true, &eh, &term_sim, &early, &last) != fmi3OK) return 2;
        current = term_sim ? last : current + step_now;
        get(i, ov, 32, ovv, 32);
        for (int k = 0; k < 32; ++k) fprintf(out, "%.9f,", ovv[k]);
        fprintf(out, "%d\n", term_sim ? 1 : 0);
        if (term_sim) break;
    }
    fclose(out);
    term(i); freei(i); dlclose(h);
    snprintf(cmd, sizeof(cmd), "rm -rf '%s'", dir); system(cmd);
    return 0;
}
