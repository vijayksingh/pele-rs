# Profile execution

Parent confirmed both sequential VM18 jobs finished (session18501 exit0) before authorizing the sole profile run. From `/home/admin/pele-perf.jL9wzP`:

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/profile-native-pod-sankey-c9b8.mjs --control parse-architecture/native-canonical-2310934/control-package --candidate parse-architecture/native-pod-33224aa/candidate-package --output parse-architecture/native-pod-33224aa/profiles
```

Upload70926 exit0; execution51842 exit0; profile fetch38592 exit0; executed-harness fetch90977 exit0. No failed launch, installation, extra VM workload or production edit. Six windows used default2s duration,300ms warmup,250µs sampling. Native/control storage probes and exact outputs passed outside the sampling windows. Complete package maps were unchanged after profiling.
