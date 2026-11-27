/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: AGPL-3.0-or-later
 * Independent FMI 3.0 checkpoint/resume research model (no Poieweave linkage).
 */
#include "fmi3.h"
#include <math.h>
#include <stdlib.h>
#include <string.h>

const char *fmi3GetVersion(void) { return "3.0"; }

typedef struct {
    fmi3Float64 v[128];
    double time;
    int phase; /* 0 initial, 1 checkpointed, 2 resumed, 3 terminal */
    double end_b;
    int ev_count;
} Instance;

static double val(Instance *i, int vr) { return i->v[vr]; }
static void set(Instance *i, int vr, double x) { i->v[vr] = x; }
static void event(Instance *i, int kind) { if (i->ev_count < 8) { set(i, 50+i->ev_count, kind); set(i, 60+i->ev_count, i->time); i->ev_count++; } set(i,40,i->ev_count); set(i,41,kind); set(i,42,i->time); }

fmi3Instance fmi3InstantiateCoSimulation(const char *a, const char *b, const char *c, fmi3Boolean d, fmi3Boolean e,
    fmi3Boolean f, fmi3Boolean g, const char *const *h, size_t n, fmi3InstanceEnvironment env,
    fmi3LogMessageCallback log, fmi3IntermediateUpdateCallback iu) {
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)h;(void)n;(void)env;(void)log;(void)iu;
    Instance *i = calloc(1, sizeof(Instance)); return i;
}
fmi3Status fmi3EnterInitializationMode(fmi3Instance i, fmi3Boolean a, fmi3Float64 b, fmi3Float64 c, fmi3Boolean d, fmi3Float64 e) {
    (void)i;(void)a;(void)b;(void)c;(void)d;(void)e; return fmi3OK;
}
fmi3Status fmi3ExitInitializationMode(fmi3Instance i) { return i ? fmi3OK : fmi3Error; }
fmi3Status fmi3SetFloat64(fmi3Instance i, const fmi3ValueReference vr[], size_t n, const fmi3Float64 v[], size_t m) {
    if (!i || m < n) return fmi3Error; Instance *s = i; for (size_t k=0;k<n;k++) if (vr[k] < 128) s->v[vr[k]] = v[k]; return fmi3OK;
}
fmi3Status fmi3GetFloat64(fmi3Instance i, const fmi3ValueReference vr[], size_t n, fmi3Float64 v[], size_t m) {
    if (!i || m < n) return fmi3Error; Instance *s = i; for (size_t k=0;k<n;k++) v[k] = vr[k] < 128 ? s->v[vr[k]] : 0.0; return fmi3OK;
}
fmi3Status fmi3DoStep(fmi3Instance i, fmi3Float64 current, fmi3Float64 step, fmi3Boolean a, fmi3Boolean *eh,
    fmi3Boolean *term, fmi3Boolean *early, fmi3Float64 *last) {
    (void)a; if (!i) return fmi3Error; Instance *s = i;
    if (s->phase == 0) {
        double t1 = val(s,3) > 0 ? val(s,3) : 7200.0;
        double t2 = val(s,4) > 0 ? val(s,4) : 10800.0;
        double fail = val(s,5) >= 0 ? val(s,5) : 9000.0;
        double target = current + step;
        double next = target;
        if (fail > current + 1e-9 && fail < next) next = fail;
        if (t1 > current + 1e-9 && t1 < next) next = t1;
        if (t2 > current + 1e-9 && t2 < next) next = t2;
        s->time = next;
        if (s->time >= t1 - 1e-9) set(s,25,1);
        if (s->time >= fail - 1e-9 && fail < t2 - 1e-9) {
            set(s,23,1); set(s,24,fail); set(s,26,1); set(s,22,1); set(s,27,2); event(s,1); event(s,7); s->phase = 1;
        } else if (s->time >= fmax(t1,t2) - 1e-9) { set(s,20,1); s->phase = 3; }
    } else if (s->phase == 1) {
        s->time = current;
        if (val(s,12) >= 0.5) {
#ifdef MUT_IGNORE_AUTH
            int authorized = 1;
#else
            int authorized = val(s,10) >= 0.5;
#endif
#ifdef MUT_FORGOT_FAILED
            int proposal_ok = 1;
#else
            int proposal_ok = val(s,11) >= 1.5;
#endif
            double candidate = val(s,31);
            int candidate_ok = ((int)candidate == 2 || (int)candidate == 3);
            event(s,3);
            if (!authorized || !proposal_ok || !candidate_ok) { set(s,30,1); set(s,33,0); set(s,34, authorized ? (proposal_ok ? 1 : 3) : 2); s->phase = 3; }
            else { event(s,4); event(s,5); event(s,6); set(s,32,candidate); set(s,33,1); double start = val(s,35) > current ? val(s,35) : current; double duration = val(s,36) > 0 ? val(s,36) : (val(s,4) > 0 ? val(s,4) : 10800.0); set(s,37,start); s->end_b = start + duration; set(s,29,1);
#ifdef MUT_FORGOT_FAILED
                set(s,27, val(s,11) >= 1.5 ? 2 : 1);
#endif
#ifdef MUT_REPLAY_A
                set(s,25,0);
#endif
                s->phase = 2; }
        }
    } else if (s->phase == 2) {
        double target = current + step; if (target > s->end_b) target = s->end_b; s->time = target;
        double start = val(s,37);
        if (s->time >= start - 1e-9) { set(s,60+5, start); set(s,60+6, start); }
        if (s->time >= s->end_b - 1e-9) { set(s,20,1); set(s,28,1); set(s,38,s->end_b); event(s,9); event(s,8); s->phase = 3; }
    } else s->time = current;
    if (term) *term = (s->phase == 3); if (eh) *eh = 0; if (early) *early = 0; if (last) *last = s->time;
    return fmi3OK;
}
fmi3Status fmi3Terminate(fmi3Instance i) { (void)i; return fmi3OK; }
fmi3Status fmi3FreeInstance(fmi3Instance i) { free(i); return fmi3OK; }
fmi3Status fmi3GetString(fmi3Instance i, const fmi3ValueReference vr[], size_t n, const char *v[], size_t m) {
    (void)i;(void)vr;(void)n;(void)v;(void)m; return fmi3Error;
}
fmi3Status fmi3Reset(fmi3Instance i) { return i ? fmi3OK : fmi3Error; }
fmi3Status fmi3SetDebugLogging(fmi3Instance i, fmi3Boolean on, size_t n, const fmi3ValueReference vr[]) { (void)i;(void)on;(void)n;(void)vr; return fmi3OK; }
