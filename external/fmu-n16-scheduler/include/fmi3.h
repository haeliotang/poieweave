/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef POIE_FMI3_MINIMAL_H
#define POIE_FMI3_MINIMAL_H
#include <stdbool.h>
#include <stddef.h>

typedef void *fmi3Instance;
typedef void *fmi3InstanceEnvironment;
typedef void *fmi3FMUState;
typedef bool fmi3Boolean;
typedef double fmi3Float64;
typedef unsigned int fmi3ValueReference;
typedef int fmi3Status;
typedef void (*fmi3LogMessageCallback)(fmi3InstanceEnvironment, fmi3Status, const char *, const char *, ...);
typedef void (*fmi3IntermediateUpdateCallback)(fmi3InstanceEnvironment, fmi3Float64, fmi3Boolean, fmi3Boolean,
                                               fmi3Boolean, fmi3Float64, fmi3Boolean, const fmi3ValueReference *,
                                               size_t, const fmi3Float64 *, size_t);

#define fmi3OK 0
#define fmi3Warning 1
#define fmi3Discard 2
#define fmi3Error 3
#define fmi3Fatal 4

#endif
