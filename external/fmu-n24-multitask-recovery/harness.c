/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: AGPL-3.0-or-later */
#include "fmi3.h"
#include <dlfcn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
typedef const char *(*gv_t)(void);
typedef fmi3Instance (*inst_t)(const char*,const char*,const char*,fmi3Boolean,fmi3Boolean,fmi3Boolean,fmi3Boolean,const char*const*,size_t,fmi3InstanceEnvironment,fmi3LogMessageCallback,fmi3IntermediateUpdateCallback);
typedef fmi3Status (*init_t)(fmi3Instance,fmi3Boolean,fmi3Float64,fmi3Float64,fmi3Boolean,fmi3Float64);
typedef fmi3Status (*exit_t)(fmi3Instance);
typedef fmi3Status (*step_t)(fmi3Instance,fmi3Float64,fmi3Float64,fmi3Boolean,fmi3Boolean*,fmi3Boolean*,fmi3Boolean*,fmi3Float64*);
typedef fmi3Status (*set_t)(fmi3Instance,const fmi3ValueReference[],size_t,const fmi3Float64[],size_t);
typedef fmi3Status (*get_t)(fmi3Instance,const fmi3ValueReference[],size_t,fmi3Float64[],size_t);
typedef fmi3Status (*free_t)(fmi3Instance);
static double getv(get_t g,fmi3Instance i,int vr){fmi3ValueReference r=vr;double v=0;g(i,&r,1,&v,1);return v;}
static void setv(set_t s,fmi3Instance i,int vr,double x){fmi3ValueReference r=vr;s(i,&r,1,&x,1);}
int main(int argc,char**argv){if(argc<3||argc>14)return 2;int use_auth=argc<4?1:atoi(argv[3]);int proposal=argc<5?2:atoi(argv[4]);double t1=argc<6?7200.0:atof(argv[5]);double t2=argc<7?10800.0:atof(argv[6]);double fail=argc<8?9000.0:atof(argv[7]);int candidate=argc<9?2:atoi(argv[8]);double start=argc<10?9000.0:atof(argv[9]);double duration=argc<11?10800.0:atof(argv[10]);double cf=argc<12?0.0:atof(argv[11]);int ctarget=argc<13?1:atoi(argv[12]);double cdur=argc<14?0.0:atof(argv[13]);char dir[]="/tmp/poie-n22-XXXXXX";if(!mkdtemp(dir))return 2;char cmd[4096];snprintf(cmd,sizeof cmd,"unzip -q '%s' -d '%s'",argv[1],dir);if(system(cmd))return 2;
#if defined(__APPLE__)
 const char*libname="poie_n24_multitask_recovery.dylib";const char*platform="darwin";
#else
 const char*libname="poie_n24_multitask_recovery.so";const char*platform="linux";
#endif
#if defined(__aarch64__)||defined(__arm64__)
 const char*arch="aarch64";
#else
 const char*arch="x86_64";
#endif
 char lib[4096];snprintf(lib,sizeof lib,"%s/binaries/%s-%s/%s",dir,arch,platform,libname);void*h=dlopen(lib,RTLD_NOW);if(!h){fprintf(stderr,"%s\n",dlerror());return 2;}
 gv_t gv=(gv_t)dlsym(h,"fmi3GetVersion");inst_t inst=(inst_t)dlsym(h,"fmi3InstantiateCoSimulation");init_t enter=(init_t)dlsym(h,"fmi3EnterInitializationMode");exit_t exitm=(exit_t)dlsym(h,"fmi3ExitInitializationMode");step_t ds=(step_t)dlsym(h,"fmi3DoStep");set_t set=(set_t)dlsym(h,"fmi3SetFloat64");get_t get=(get_t)dlsym(h,"fmi3GetFloat64");free_t fr=(free_t)dlsym(h,"fmi3FreeInstance");
 fmi3Instance i=inst("n22","",NULL,0,0,0,0,NULL,0,NULL,NULL,NULL);enter(i,0,0,0,0,0);exitm(i);
 setv(set,i,3,t1);setv(set,i,4,t2);setv(set,i,5,fail);setv(set,i,10,0);setv(set,i,11,proposal);setv(set,i,12,0);
 double t=0;for(int k=0;k<100&&getv(get,i,23)<0.5;k++){fmi3Boolean eh=0,term=0,early=0;fmi3Float64 last=0;ds(i,t,1000,0,&eh,&term,&early,&last);t=last;}
 double cp=getv(get,i,24),ca=getv(get,i,25),ub=getv(get,i,26);
 setv(set,i,10,use_auth);setv(set,i,11,proposal);setv(set,i,31,candidate);setv(set,i,35,start);setv(set,i,36,duration);setv(set,i,39,cf);setv(set,i,47,ctarget);setv(set,i,36,duration);setv(set,i,49,cdur);setv(set,i,39,cf);setv(set,i,40,ctarget);setv(set,i,12,1);
 for(int k=0;k<100&&getv(get,i,20)<0.5;k++){fmi3Boolean eh=0,term=0,early=0;fmi3Float64 last=0;ds(i,t,1000,0,&eh,&term,&early,&last);t=last;}
 double comp=getv(get,i,20),f1=getv(get,i,22),res=getv(get,i,28),resu=getv(get,i,29),rej=getv(get,i,30),rb=getv(get,i,27);
 double ca_final=getv(get,i,25),cp_final=getv(get,i,24);double actual_start=getv(get,i,37),actual_completion=getv(get,i,38),second_int=getv(get,i,44),irrel=getv(get,i,45),pre=getv(get,i,48);double exec=getv(get,i,32),acc=getv(get,i,33),rr=getv(get,i,34),ec=getv(get,i,40);double ek[8],et[8];for(int k=0;k<8;k++){ek[k]=getv(get,i,50+k);et[k]=getv(get,i,60+k);}
 printf("{\"taskBStart\":%.0f,\"taskBEnd\":%.0f,\"taskCStart\":%.0f,\"taskCEnd\":%.0f,\"completedB\":%.0f,\"completedC\":%.0f,\"completed\":%.0f,\"failedRobotM1\":%.0f,\"checkpointAt_s\":%.0f,\"completedA\":%.0f,\"unfinishedB\":%.0f,\"taskBRobot\":%.0f,\"restored\":%.0f,\"resumed\":%.0f,\"rejected\":%.0f,\"executedRobot\":%.0f,\"candidateAccepted\":%.0f,\"rejectionReason\":%.0f,\"eventCount\":%.0f,\"actualStart_s\":%.0f,\"actualCompletion_s\":%.0f,\"secondInterrupted\":%.0f,\"irrelevantFailureObserved\":%.0f,\"preStartInvalidated\":%.0f,\"eventKinds\":[%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.0f],\"eventTimes\":[%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.0f],\"survived_s\":%.0f}\n" ,getv(get,i,101),getv(get,i,102),getv(get,i,103),getv(get,i,104),getv(get,i,105),getv(get,i,100),comp,f1,cp_final,ca_final,ub,rb,res,resu,rej,exec,acc,rr,ec,actual_start,actual_completion,second_int,irrel,pre,ek[0],ek[1],ek[2],ek[3],ek[4],ek[5],ek[6],ek[7],et[0],et[1],et[2],et[3],et[4],et[5],et[6],et[7],t);
 fr(i);return 0;}
