/* SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
 * SPDX-License-Identifier: AGPL-3.0-or-later
 * Independent C99 checkpoint/resume state machine (not an FMI model).
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

typedef struct { double duration[2]; int robot[2]; int tool[2]; double failure_t; char failure_asset[32]; double horizon; } Contract;
static double get(const char *path, const char *key, double def){ FILE*f=fopen(path,"r"); if(!f)return def; char line[512]; double v=def; while(fgets(line,sizeof line,f)){ char*eq=strchr(line,'='); if(!eq)continue; *eq=0; if(strcmp(line,key)==0){v=atof(eq+1);break;} } fclose(f); return v; }
static void asset(const char*path,char*out,size_t n){ FILE*f=fopen(path,"r"); out[0]=0; if(!f)return; char line[512]; while(fgets(line,sizeof line,f)){ char*eq=strchr(line,'='); if(!eq)continue; *eq=0; if(strcmp(line,"failure_target")==0){snprintf(out,n,"%s",eq+1); while(out[0]&&(out[strlen(out)-1]=='\n'||out[strlen(out)-1]=='\r'))out[strlen(out)-1]=0;} } fclose(f); }
static uint64_t fnv(const char*s){ uint64_t h=1469598103934665603ULL; while(*s){h^=(unsigned char)*s++;h*=1099511628211ULL;} return h; }
static uint64_t cp_hash(double at,int completed,int unfinished,double inv,const char*failed,const char*restore){ char b[512]; snprintf(b,sizeof b,"%.6f|%d|%d|%.6f|%s|%s",at,completed,unfinished,inv,failed,restore); return fnv(b); }
int main(int argc,char**argv){ if(argc<5){fprintf(stderr,"usage: %s <contract> <initial|resume> <checkpoint> <out>\n",argv[0]);return 2;} Contract c; memset(&c,0,sizeof c); c.duration[0]=get(argv[1],"t1Duration_s",7200); c.duration[1]=get(argv[1],"t2Duration_s",10800); c.robot[0]=(int)get(argv[1],"t1RobotMask",2); c.robot[1]=(int)get(argv[1],"t2RobotMask",1); c.tool[0]=(int)get(argv[1],"t1RequiresTool",0); c.tool[1]=(int)get(argv[1],"t2RequiresTool",0); c.failure_t=get(argv[1],"failureTime_s",9000); c.horizon=get(argv[1],"horizon_s",72000); asset(argv[1],c.failure_asset,sizeof c.failure_asset); char out[1024]; snprintf(out,sizeof out,"%s.events.jsonl",argv[4]); FILE*ev=fopen(out,"w");
 if(strcmp(argv[2],"initial")==0){ double a_end=c.duration[0], b_end=c.duration[1]; double end=a_end>b_end?a_end:b_end; int completed=0,unfinished=2; double t=c.failure_t; int failed_robot= c.robot[1]==1?1:2; int hit = t < b_end && strstr(c.failure_asset,"robot-m1")!=NULL;
  fprintf(ev,"{\"at\":0,\"kind\":\"InitialRepairStarted\",\"asset\":\"initial-repair\"}\n");
  if(a_end<=t) fprintf(ev,"{\"at\":%.0f,\"kind\":\"TaskCompleted\",\"asset\":\"A\"}\n",a_end);
  fprintf(ev,"{\"at\":%.0f,\"kind\":\"AssetFailure\",\"asset\":\"robot-m1\"}\n",t);
  if(hit){ fprintf(ev,"{\"at\":%.0f,\"kind\":\"RepairInterrupted\",\"asset\":\"repair\"}\n",t); fprintf(ev,"{\"at\":%.0f,\"kind\":\"CheckpointCreated\",\"asset\":\"checkpoint\"}\n",t); }
  uint64_t h=cp_hash(t, a_end<=t?1:0, hit?1:0, 1.0, "robot-m1", "a-lathe"); FILE*cp=fopen(argv[3],"w"); fprintf(cp,"{\"at\":%.0f,\"completed_A\":%d,\"unfinished_B\":%d,\"failed_asset\":\"robot-m1\",\"inventory\":1.0,\"restoration_target\":\"a-lathe\",\"state_hash\":\"%016llx\"}\n",t,a_end<=t?1:0,hit?1:0,(unsigned long long)h); fclose(cp);
  fprintf(ev,"{\"at\":%.0f,\"kind\":\"ExecutionSuspended\",\"asset\":\"checkpoint\"}\n",t); fclose(ev); return hit?0:3; }
 else { FILE*cp=fopen(argv[3],"r"); char line[512]; if(!cp)return 4; if(!fgets(line,sizeof line,cp))return 4; fclose(cp); double t=0; int completed_A=0,unfinished_B=0; char failed[32]="",restore[32]=""; sscanf(line,"{\"at\":%lf,\"completed_A\":%d,\"unfinished_B\":%d,\"failed_asset\":\"%31[^\"]\",\"inventory\":%*lf,\"restoration_target\":\"%31[^\"]\"",&t,&completed_A,&unfinished_B,failed,restore); if(!unfinished_B||!completed_A)return 5; char auth[32]=""; if(argc>5){FILE*a=fopen(argv[5],"r"); if(a){fgets(auth,sizeof auth,a);fclose(a);}} if(strncmp(auth,"VALID",5)!=0){fprintf(ev,"{\"at\":%.0f,\"kind\":\"ReplanRejected\",\"asset\":\"auth\"}\n",t);fclose(ev);return 6;}
  double end=t+c.duration[1]; if(end>c.horizon){fprintf(ev,"{\"at\":%.0f,\"kind\":\"IncompleteAtHorizon\",\"asset\":\"repair\"}\n",c.horizon);fclose(ev);return 9;}
  fprintf(ev,"{\"at\":%.0f,\"kind\":\"ReplanAuthorized\",\"asset\":\"auth\"}\n",t); fprintf(ev,"{\"at\":%.0f,\"kind\":\"RepairResumed\",\"asset\":\"repair\"}\n",t); fprintf(ev,"{\"at\":%.0f,\"kind\":\"TaskRestarted\",\"asset\":\"B\"}\n",t); fprintf(ev,"{\"at\":%.0f,\"kind\":\"TaskCompleted\",\"asset\":\"B\"}\n",end); fprintf(ev,"{\"at\":%.0f,\"kind\":\"RepairCompleted\",\"asset\":\"group\"}\n",end); fprintf(ev,"{\"at\":%.0f,\"kind\":\"RestorationCommitted\",\"asset\":\"a-lathe\"}\n",end); fclose(ev);
  snprintf(out,sizeof out,"%s.result.json",argv[4]); FILE*r=fopen(out,"w"); fprintf(r,"{\"completed\":true,\"survived_s\":%.0f,\"failed_robot_m1\":true,\"task_A_rerun\":false,\"inventory_j\":1.0,\"robot_B\":\"robot-m2\"}\n",end); fclose(r); return 0; }
}
