# Authorized native canonical profile run

Only one VM CPU-profile job ran. Parent confirmed the preceding VM18 latency job had finished and the VM slot was free. No first-use benchmark, browser installation or production edit was performed.

From `/home/admin/pele-perf.jL9wzP`:

```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node scripts/perf/profile-native-canonical-sankey-3771.mjs --control parse-architecture/native-canonical-2310934/control-package --candidate parse-architecture/native-canonical-2310934/candidate-package --output parse-architecture/native-canonical-2310934/profiles
```

Upload72309 exit0; execution58875 exit0; profile fetch86666 exit0; executed-harness fetch70645 exit0. All six planned windows completed and summary succeeded. No failed launch attempt occurred. The fetched harness is byte-identical to the recorded harness hash. Both full113-file package maps match the preceding benchmark and were unchanged after profiling. Browser/host/timestamps and per-window hashes are in the original summary.

Raw profile filenames preserve the exact operation/runtime labels. The compressed files decompress to the original raw hashes. The run used six2s windows with300ms warmup and250us sampling. Backend and storage checks are outside the CDP windows; successful output equality is recorded before/after each stage.
