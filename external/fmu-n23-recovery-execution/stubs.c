/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: Apache-2.0
 *
 * Explicit unsupported-interface stubs required by generic FMI 3.0 importers.
 * The FMU is a Co-Simulation FMU; unsupported API calls fail closed.
 */
#include "fmi3.h"

fmi3Status fmi3ActivateModelPartition(void) { return fmi3Error; }

fmi3Status fmi3CompletedIntegratorStep(void) { return fmi3Error; }

fmi3Status fmi3DeserializeFMUState(void) { return fmi3Error; }

fmi3Status fmi3EnterConfigurationMode(void) { return fmi3Error; }

fmi3Status fmi3EnterContinuousTimeMode(void) { return fmi3Error; }

fmi3Status fmi3EnterEventMode(void) { return fmi3Error; }

fmi3Status fmi3EnterStepMode(fmi3Instance instance) { (void)instance; return fmi3OK; }

fmi3Status fmi3EvaluateDiscreteStates(void) { return fmi3Error; }

fmi3Status fmi3ExitConfigurationMode(void) { return fmi3Error; }

fmi3Status fmi3FreeFMUState(void) { return fmi3Error; }

fmi3Status fmi3GetAdjointDerivative(void) { return fmi3Error; }

fmi3Status fmi3GetBinary(void) { return fmi3Error; }

fmi3Status fmi3GetBoolean(void) { return fmi3Error; }

fmi3Status fmi3GetClock(void) { return fmi3Error; }

fmi3Status fmi3GetContinuousStateDerivatives(void) { return fmi3Error; }

fmi3Status fmi3GetContinuousStates(void) { return fmi3Error; }

fmi3Status fmi3GetDirectionalDerivative(void) { return fmi3Error; }

fmi3Status fmi3GetEventIndicators(void) { return fmi3Error; }

fmi3Status fmi3GetFMUState(void) { return fmi3Error; }

fmi3Status fmi3GetFloat32(void) { return fmi3Error; }

fmi3Status fmi3GetInt16(void) { return fmi3Error; }

fmi3Status fmi3GetInt32(void) { return fmi3Error; }

fmi3Status fmi3GetInt64(void) { return fmi3Error; }

fmi3Status fmi3GetInt8(void) { return fmi3Error; }

fmi3Status fmi3GetIntervalDecimal(void) { return fmi3Error; }

fmi3Status fmi3GetIntervalFraction(void) { return fmi3Error; }

fmi3Status fmi3GetNominalsOfContinuousStates(void) { return fmi3Error; }

fmi3Status fmi3GetNumberOfContinuousStates(void) { return fmi3Error; }

fmi3Status fmi3GetNumberOfEventIndicators(void) { return fmi3Error; }

fmi3Status fmi3GetNumberOfVariableDependencies(void) { return fmi3Error; }

fmi3Status fmi3GetOutputDerivatives(void) { return fmi3Error; }

fmi3Status fmi3GetShiftDecimal(void) { return fmi3Error; }

fmi3Status fmi3GetShiftFraction(void) { return fmi3Error; }


fmi3Status fmi3GetUInt16(void) { return fmi3Error; }

fmi3Status fmi3GetUInt32(void) { return fmi3Error; }

fmi3Status fmi3GetUInt64(void) { return fmi3Error; }

fmi3Status fmi3GetUInt8(void) { return fmi3Error; }

fmi3Status fmi3GetVariableDependencies(void) { return fmi3Error; }

fmi3Status fmi3InstantiateModelExchange(void) { return fmi3Error; }

fmi3Status fmi3InstantiateScheduledExecution(void) { return fmi3Error; }

fmi3Status fmi3SerializeFMUState(void) { return fmi3Error; }

fmi3Status fmi3SerializedFMUStateSize(void) { return fmi3Error; }

fmi3Status fmi3SetBinary(void) { return fmi3Error; }

fmi3Status fmi3SetBoolean(void) { return fmi3Error; }

fmi3Status fmi3SetClock(void) { return fmi3Error; }

fmi3Status fmi3SetContinuousStates(void) { return fmi3Error; }

fmi3Status fmi3SetFMUState(void) { return fmi3Error; }

fmi3Status fmi3SetFloat32(void) { return fmi3Error; }

fmi3Status fmi3SetInt16(void) { return fmi3Error; }

fmi3Status fmi3SetInt32(void) { return fmi3Error; }

fmi3Status fmi3SetInt64(void) { return fmi3Error; }

fmi3Status fmi3SetInt8(void) { return fmi3Error; }

fmi3Status fmi3SetIntervalDecimal(void) { return fmi3Error; }

fmi3Status fmi3SetIntervalFraction(void) { return fmi3Error; }

fmi3Status fmi3SetShiftDecimal(void) { return fmi3Error; }

fmi3Status fmi3SetShiftFraction(void) { return fmi3Error; }

fmi3Status fmi3SetString(void) { return fmi3Error; }

fmi3Status fmi3SetTime(fmi3Instance instance, fmi3Float64 time) { (void)instance; (void)time; return fmi3OK; }

fmi3Status fmi3SetUInt16(void) { return fmi3Error; }

fmi3Status fmi3SetUInt32(void) { return fmi3Error; }

fmi3Status fmi3SetUInt64(void) { return fmi3Error; }

fmi3Status fmi3SetUInt8(void) { return fmi3Error; }

fmi3Status fmi3UpdateDiscreteStates(void) { return fmi3Error; }
