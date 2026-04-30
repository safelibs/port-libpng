## 1. High-level architecture

This tree is a Rust implementation of the libpng 1.6.43 ABI packaged as the
Ubuntu/Debian `libpng1.6` source package. `safe/Cargo.toml` defines package
`libpng-safe` version `1.6.43`, edition `2024`, with library target `png16`
and crate types `cdylib` and `staticlib`. `cargo metadata --locked
--format-version 1 --no-deps --manifest-path safe/Cargo.toml` reports no
feature table and the direct dependencies `flate2 = "1"`, `libc = "0.2"`,
`png = "0.18.1"`, plus build dependency `cc = "1.2"`.

The public libpng install and ABI surface is frozen by checked-in files:

| Surface | Evidence |
| --- | --- |
| Public headers | `safe/include/png.h`, `safe/include/pngconf.h`, `safe/include/pnglibconf.h` |
| Intended exports | `safe/abi/exports.txt`, 246 `png_*` symbols |
| Linux symbol versioning | `safe/abi/libpng.vers`, global version `PNG16_0` |
| Install layout | `safe/abi/install-layout.txt` |
| pkg-config/config scripts | `safe/pkg/libpng.pc.in`, `safe/pkg/libpng-config.in` |

`safe/tools/check-exports.sh` rebuilds and stages the library, extracts
dynamic symbols with `readelf`, extracts versioned symbols with `objdump`,
diffs both against `safe/abi/exports.txt`, verifies every frozen export is
versioned as `PNG16_0`, and rejects an export count other than 246.

`safe/src/lib.rs` wires the implementation modules:

| Area | Source files |
| --- | --- |
| Public C ABI shims | `safe/src/common.rs`, `safe/src/memory.rs`, `safe/src/error.rs`, `safe/src/io.rs`, `safe/src/get.rs`, `safe/src/set.rs`, `safe/src/read_transform.rs`, `safe/src/read_progressive.rs`, `safe/src/compat_exports.rs`, `safe/src/write.rs`, `safe/src/write_util.rs`, `safe/src/write_transform.rs`, `safe/src/colorspace.rs`, `safe/src/interlace.rs`, `safe/src/simplified.rs` |
| Internal state | `safe/src/state.rs` owns `PngStructState` and `PngInfoState`; `safe/src/state.rs:267` and `safe/src/state.rs:272` store registries in `OnceLock<Mutex<HashMap<...>>>`, keyed by exported opaque `png_struct` and `png_info` pointer values via `NonZeroUsize` at `safe/src/state.rs:262`, `safe/src/state.rs:265`, `safe/src/state.rs:306`, and `safe/src/state.rs:310` |
| Read-side parsing and transitions | `safe/src/read.rs`, `safe/src/read_util.rs`, `safe/src/chunks.rs`, `safe/src/read_progressive.rs`, `safe/src/bridge_ffi.rs` |
| Write-side behavior | `safe/src/write.rs`, `safe/src/write_runtime.rs`, `safe/src/write_util.rs`, `safe/src/write_transform.rs`, `safe/src/bridge_ffi.rs`; the write runtime uses Rust `png` and `flate2` building blocks |
| Simplified `png_image_*` APIs | `safe/src/simplified.rs`, `safe/src/simplified_runtime.rs`, plus bridge helpers in `safe/src/bridge_ffi.rs` |
| Compatibility exports | `safe/src/compat_exports.rs` |
| Read/write transforms | `safe/src/read_transform.rs`, `safe/src/write_transform.rs` |
| Colorspace and interlace | `safe/src/colorspace.rs`, `safe/src/interlace.rs` |
| zlib interop | `safe/src/zlib.rs` |
| I/O, memory, and error handling | `safe/src/io.rs`, `safe/src/memory.rs`, `safe/src/error.rs` |

ABI panic containment is centralized in `safe/src/common.rs:121`
(`abi_guard!`) and `safe/src/common.rs:137` (`abi_guard_no_png!`).
Checked-in C boundary files remain as documentation/glue:
`safe/cshim/longjmp_bridge.c` documents the `jmp_buf` field ABI and final
longjmp callback handoff, and `safe/cshim/read_phase_bridge.c` documents
private-layout mirror types for read-side field synchronization. The old hidden
support runtime is not synthesized or compiled: `safe/tools/build_support/build_support.rs`
keeps `build_support_core()` as a no-op and states that the previous hidden
support runtime is no longer generated from vendor C sources.

Build, staging, linking, install-surface, and packaging behavior is defined by:

| File | Role |
| --- | --- |
| `safe/build.rs` | Cargo build script; emits `-lz`, `-lm`, version script, SONAME, and stages an ABI tree after static output exists |
| `safe/tools/stage-install-tree.sh` | Recreates the staged `/usr/include`, `/usr/lib/<multiarch>`, `/usr/bin`, and pkg-config/config-script surface; links `libpng16.so.16.43.0` from the Rust static library |
| `safe/pkg/libpng.pc.in` | Template for `libpng16.pc` and `libpng.pc` |
| `safe/pkg/libpng-config.in` | Template for `libpng16-config` and `libpng-config` |
| `safe/debian/control` | Debian package metadata and build inputs |
| `safe/debian/rules` | Debian build, test, install, and artifact refresh rules |

`safe/debian/control` declares exactly these build dependencies:
`debhelper-compat (= 13), cargo, dpkg-dev (>= 1.22.5), gcc, mawk, python3,
rustc, zlib1g-dev`. `safe/debian/rules` builds the Rust crate, stages
headers/libs/pkg-config/config scripts, builds `pngfix` and `png-fix-itxt`,
runs upstream and link-compatibility tests, and packages `libpng16-16t64`,
`libpng-dev`, `libpng-tools`, and `libpng16-16-udeb`.

Directory map:

| Directory | Contents |
| --- | --- |
| `safe/src` | Rust libpng ABI implementation, read/write runtimes, state registries, C ABI shims, bridge helpers, zlib interop |
| `safe/include` | Frozen libpng public headers installed under `usr/include/libpng16` and top-level compatibility links |
| `safe/abi` | Export baseline, version script, install-layout baseline |
| `safe/pkg` | `libpng.pc` and `libpng-config` templates plus source snapshot manifest |
| `safe/tools` | Build, staging, ABI, header, install-surface, upstream, CVE, dependent, and package verification scripts |
| `safe/tests` | C smoke/regression drivers for core API, read paths, transforms, upstream fixtures, CVE coverage, and dependent API patterns |
| `safe/cshim` | Checked-in C boundary documentation/glue for `jmp_buf` and private-layout bridge fields |
| `safe/contrib` | Upstream libpng tools, libtests, pngsuite, and testpng inputs carried in the safe source package |
| `safe/debian` | Debian packaging, patches, install manifests, maintainer scripts/metadata, and existing untracked build artifacts |

## 2. Where the unsafe Rust lives

The authoritative unsafe inventory is the tracked Rust/source inventory from:

```bash
git ls-files safe/src safe/build.rs safe/cshim safe/tools/build_support/build_support.rs \
  | xargs rg -n '\bunsafe\b|unsafe extern "C"|#\[unsafe|unsafe impl'
```

That command produced 1,576 tracked implementation matches. The required
full-tree search,

```bash
grep -RIn '\bunsafe\b' safe >/tmp/libpng-safe-unsafe-full.txt
```

produced 4,101 lines and also traversed generated Cargo registry sources,
Debian build outputs, upstream prose such as `safe/libpng.3`, comments,
Debian patch history, and symlinked/binary package artifacts. Those full-tree
matches are useful for reconciliation, but the tracked Rust source inventory
below is the live implementation inventory.

Grouped unsafe surfaces:

| Group | File:line evidence | Why it is unsafe |
| --- | --- | --- |
| Public libpng ABI entry points and panic guards | `safe/src/common.rs:121`, `safe/src/common.rs:137`; public `png_*` exports in `safe/src/common.rs:213,241,246,251,256,261,266,315,322,333,383,402,407`, `safe/src/memory.rs:266,276,286,296,306,313,320,336,344,363,380,399,416,427,446,465,481,522,562`, `safe/src/error.rs:50,60,71,87,97,108,118,134,142,181`, `safe/src/io.rs:68,76,94,110,127,139,151,173,181,195,203,215,227,243,251,256`, `safe/src/get.rs:430,446,455,464,473,482` plus macro-generated getters at `safe/src/get.rs:216-218`, `safe/src/set.rs:222,229,409,424,437,450,503,513,526` plus macro-generated setters at `safe/src/set.rs:202-203`, `safe/src/read_transform.rs:72,79,86,93,100,107,114,121,155,169,176,183,190`, `safe/src/read_progressive.rs:439,449`, `safe/src/compat_exports.rs:365,385,423,430,437,444,459,484,494,504,514,529,543,553,563,624,629,634,654,681,694,707,720,733,752,765,778,791,804,818,842,854,876,905,918,956,974,984,994,1004,1014,1024,1034,1044,1070,1084,1095,1112`, `safe/src/write.rs:81,91,98,105,116,123,130,142,152`, `safe/src/write_util.rs:6,15,22,34,45,56,63,73,83,96,109,122,135,148,161,174,187`, `safe/src/write_transform.rs:9,21,41,61,73,85,93,101`, `safe/src/colorspace.rs:441,453,467,486,506,517,526,551,602,623`, `safe/src/interlace.rs:83`, `safe/src/simplified.rs:281,295,309,324,341,346,371,389` | These functions are exported or macro-generated C ABI entry points. They accept foreign pointers, callback pointers, and libpng ABI structs from C callers; guards contain Rust panics before returning or aborting across the ABI. |
| Internal `bridge_` and `png_safe_` ABI shims | `safe/src/bridge_ffi.rs:2718,2729,2740,2751,2762,2770,2886,2889,2897,2906,2920,2949,2954,2961,2971,2981,2999,3047,3095,3113,3135,3151,3167,3185,3228,3253,3268,3293,3315,3333,3349,3379,3388,3416,3434,3482,3512,3542,3568,3573,3584,3596,3606,3616,3623,3643,3658,3687,3728,3744,3760,3769,3783,3799,3850,3864,3878,3892,3910,3931,3946,3955,3977,3986,4069,4084,4108,4144,4180,4237,4250,4310,4335,4346,4354,4363,4410,4510,4513,4524,4536,4547,4558,4566,4571,4576,4585,4590,4616,4680,4687,4697,4706,4716,4744,4752,4760,4776,4799,4807,4817,4827,4845,4868,4881,4894,4907,4915,4923,4932,4945,4966,4987,5010`; `safe/src/io.rs:318,323,328,337,347,355,363,374,379,388,393,401,409,426,433,438`; `safe/src/error.rs:204,212,220,228,236,244,252,262,267`; `safe/src/read.rs:1821,1854,1884,1908,1950,1995,2007`; `safe/src/read_progressive.rs:25,404` | These are internal C-callable bridge names and Rust-to-Rust extern shims. They preserve existing bridge call structure and C interop but are not all strictly required by the public libpng ABI. |
| Raw pointer reads/writes and C string/slice conversion | Dense matches in `safe/src/bridge_ffi.rs`, `safe/src/read.rs`, `safe/src/simplified.rs`, `safe/src/simplified_runtime.rs`, `safe/src/get.rs`, `safe/src/set.rs`, `safe/src/common.rs`, and `safe/src/compat_exports.rs`; complete tracked line list is in the file-by-file table below | These convert `png_*p` arguments into Rust references, slices, strings, row pointers, chunk buffers, and output pointer writes after null and size checks. |
| Callback invocation into application-provided C functions | `safe/src/error.rs:31,38,45`, `safe/src/io.rs:5,22,39,53`, `safe/src/chunks.rs:31,39,48,52`, `safe/src/read_progressive.rs:191,211,226`, `safe/src/bridge_ffi.rs:4310,4317,4692`, `safe/src/write_runtime.rs:1523,1527` | The port must call user-provided error/warning, read/write, flush, row-status, user-transform, progressive, and chunk callbacks with libpng-compatible arguments. |
| Allocator integration and opaque handle lifetime | `safe/src/memory.rs:34,46,83,131,135,139,147,155,210,234,266,276,286,296,306,313,320,336,344,363,380,399,416,427,446,465,481,522,562`; state registry lines `safe/src/state.rs:267,272,313,357` | The C ABI exposes `png_struct` and `png_info` as opaque pointers and allows user malloc/free callbacks. Rust allocates handles, registers state, moves or removes info state, and frees caller-supplied or default allocations. |
| `setjmp`/`longjmp` compatibility | `safe/src/memory.rs:23,29,31,155,181`; `safe/src/error.rs:142,181,193,195`; checked-in C boundary in `safe/cshim/longjmp_bridge.c` | Existing libpng callers rely on `png_set_longjmp_fn`, `png_longjmp`, and allocator-failure behavior during creation. |
| zlib FFI | `safe/src/zlib.rs:23,24,31,114,116,117,139,152,171` | Ancillary compressed chunks are inflated through the platform zlib ABI. |
| stdio/file/time/libc helpers | `safe/src/io.rs:5,16,22,33,39,47`; `safe/src/bridge_ffi.rs:741,747,758,762,765,769,776,3497,3502,3527,3532,3966,4005,4019,4028,4043,4050,4209,4210`; `safe/src/simplified_runtime.rs:260,264,267,271,278,2704`; `safe/src/common.rs:201,232,414` | `png_init_io`, simplified file APIs, compatibility getters/setters, string lengths, floating parsing, `memcmp`, and RFC1123 time formatting use libc-compatible behavior. |
| `unsafe impl Send` for registry state | `safe/src/state.rs:75`, `safe/src/state.rs:227` | The global `Mutex<HashMap<...>>` registries contain raw C pointers and callback pointers. The impls assert that registry mutation remains synchronized and that the raw pointer values are only ABI handles. |

Unsafe that is not strictly required by the public C ABI boundary includes:
direct zlib FFI in `safe/src/zlib.rs`, internal Rust-to-Rust `extern "C"`
shims in `safe/src/get.rs`, `safe/src/set.rs`, `safe/src/read.rs`,
`safe/src/read_progressive.rs`, `safe/src/read_transform.rs`, and
`safe/src/bridge_ffi.rs`, libc stdio/file helpers used by simplified APIs, and
the `unsafe impl Send` declarations in `safe/src/state.rs:75` and
`safe/src/state.rs:227`.

File-by-file tracked unsafe matches from `/tmp/libpng-safe-unsafe-tracked.txt`:

| File | Matched lines | Principal justification |
| --- | --- | --- |
| `safe/src/bridge_ffi.rs` | 435, 439, 498, 606, 613, 669, 691, 697, 702, 704, 705, 716, 741, 747, 758, 762, 765, 769, 776, 904, 936, 971, 1550, 1563, 1570, 2609, 2717, 2718, 2725, 2728, 2729, 2736, 2739, 2740, 2747, 2750, 2751, 2758, 2761, 2762, 2769, 2770, 2834, 2842, 2885, 2886, 2888, 2889, 2893, 2896, 2897, 2905, 2906, 2919, 2920, 2929, 2930, 2948, 2949, 2953, 2954, 2960, 2961, 2970, 2971, 2980, 2981, 2991, 2998, 2999, 3017, 3046, 3047, 3065, 3094, 3095, 3105, 3112, 3113, 3124, 3127, 3134, 3135, 3145, 3150, 3151, 3161, 3166, 3167, 3177, 3184, 3185, 3197, 3227, 3228, 3237, 3252, 3253, 3267, 3268, 3277, 3292, 3293, 3304, 3307, 3314, 3315, 3325, 3332, 3333, 3343, 3348, 3349, 3362, 3365, 3368, 3371, 3378, 3379, 3387, 3388, 3394, 3397, 3400, 3405, 3407, 3410, 3415, 3416, 3426, 3433, 3434, 3450, 3459, 3462, 3466, 3469, 3473, 3481, 3482, 3494, 3497, 3502, 3511, 3512, 3524, 3527, 3532, 3541, 3542, 3554, 3557, 3560, 3567, 3568, 3572, 3573, 3583, 3584, 3595, 3596, 3605, 3606, 3615, 3616, 3622, 3623, 3642, 3643, 3652, 3657, 3658, 3670, 3686, 3687, 3727, 3728, 3738, 3743, 3744, 3754, 3759, 3760, 3765, 3768, 3769, 3782, 3783, 3793, 3798, 3799, 3849, 3850, 3863, 3864, 3877, 3878, 3891, 3892, 3903, 3909, 3910, 3918, 3930, 3931, 3945, 3946, 3951, 3954, 3955, 3966, 3969, 3971, 3976, 3977, 3985, 3986, 3999, 4005, 4006, 4019, 4028, 4032, 4043, 4044, 4050, 4051, 4065, 4068, 4069, 4078, 4083, 4084, 4094, 4101, 4107, 4108, 4120, 4127, 4143, 4144, 4156, 4163, 4179, 4180, 4192, 4198, 4204, 4209, 4210, 4211, 4212, 4215, 4221, 4230, 4231, 4236, 4237, 4243, 4249, 4250, 4261, 4280, 4281, 4282, 4309, 4310, 4317, 4325, 4334, 4335, 4345, 4346, 4353, 4354, 4362, 4363, 4409, 4410, 4509, 4510, 4512, 4513, 4518, 4523, 4524, 4535, 4536, 4546, 4547, 4548, 4557, 4558, 4562, 4565, 4566, 4567, 4570, 4571, 4572, 4575, 4576, 4581, 4584, 4585, 4586, 4589, 4590, 4600, 4610, 4615, 4616, 4626, 4679, 4680, 4686, 4687, 4692, 4696, 4697, 4705, 4706, 4715, 4716, 4728, 4732, 4743, 4744, 4751, 4752, 4759, 4760, 4765, 4766, 4775, 4776, 4798, 4799, 4806, 4807, 4816, 4817, 4826, 4827, 4832, 4844, 4845, 4867, 4868, 4880, 4881, 4893, 4894, 4906, 4907, 4911, 4914, 4915, 4919, 4922, 4923, 4928, 4931, 4932, 4939, 4944, 4945, 4953, 4965, 4966, 4974, 4986, 4987, 4996, 5009, 5010, 5011, 5014, 5015, 5018, 5019, 5022, 5023, 5026, 5027, 5030, 5031, 5034, 5035, 5038, 5044, 5047, 5048, 5051, 5052, 5055, 5056, 5059, 5060, 5063, 5069, 5072, 5077, 5080, 5081, 5084, 5085, 5088, 5089, 5092, 5093, 5096, 5097, 5100, 5101, 5104, 5105, 5108, 5109, 5112, 5113, 5116, 5117, 5120, 5121, 5124, 5125, 5128, 5129, 5132, 5133, 5136, 5143, 5154, 5161, 5172, 5173, 5176, 5177, 5180, 5181, 5184, 5185, 5188, 5189, 5192, 5196, 5199, 5200, 5203, 5208, 5211, 5218, 5221, 5229, 5241, 5249, 5254, 5263, 5276, 5277 | Internal bridge layer, pointer/metadata copying, simplified file helpers, chunk/text/string conversion, callback calls, write/read bridge wrappers |
| `safe/src/read.rs` | 38, 313, 320, 339, 341, 343, 362, 368, 373, 394, 399, 400, 404, 410, 411, 414, 424, 429, 430, 433, 461, 500, 509, 512, 514, 527, 537, 548, 561, 578, 584, 589, 599, 606, 609, 612, 620, 626, 637, 652, 653, 657, 679, 688, 693, 697, 704, 715, 729, 737, 745, 749, 754, 773, 780, 793, 802, 810, 817, 832, 839, 844, 851, 855, 856, 861, 868, 872, 873, 878, 894, 919, 920, 924, 936, 945, 954, 964, 969, 970, 974, 986, 996, 1001, 1007, 1010, 1015, 1023, 1026, 1030, 1035, 1042, 1046, 1056, 1060, 1067, 1071, 1081, 1085, 1092, 1104, 1105, 1109, 1115, 1120, 1124, 1140, 1141, 1145, 1151, 1156, 1160, 1167, 1185, 1186, 1190, 1196, 1201, 1205, 1212, 1216, 1222, 1229, 1256, 1257, 1261, 1268, 1272, 1279, 1284, 1289, 1300, 1306, 1317, 1325, 1326, 1330, 1336, 1341, 1345, 1355, 1360, 1373, 1404, 1405, 1409, 1415, 1424, 1428, 1435, 1439, 1450, 1461, 1470, 1476, 1489, 1507, 1511, 1518, 1524, 1528, 1537, 1543, 1553, 1557, 1565, 1566, 1567, 1568, 1569, 1570, 1571, 1572, 1573, 1574, 1575, 1576, 1577, 1578, 1579, 1580, 1581, 1582, 1583, 1584, 1587, 1597, 1599, 1600, 1608, 1612, 1626, 1631, 1633, 1655, 1659, 1664, 1672, 1673, 1676, 1681, 1683, 1685, 1691, 1696, 1700, 1702, 1703, 1708, 1709, 1712, 1714, 1720, 1733, 1734, 1747, 1753, 1755, 1761, 1763, 1776, 1778, 1780, 1786, 1790, 1794, 1815, 1820, 1821, 1829, 1832, 1833, 1853, 1854, 1862, 1865, 1866, 1883, 1884, 1889, 1892, 1893, 1907, 1908, 1917, 1920, 1926, 1949, 1950, 1960, 1963, 1964, 1994, 1995, 2003, 2006, 2007, 2015 | Read parser, chunk state machine, snapshot/rollback, row output, user chunk callback and bridge calls |
| `safe/src/compat_exports.rs` | 64, 75, 96, 103, 151, 159, 161, 173, 191, 226, 232, 235, 238, 241, 244, 247, 250, 254, 258, 262, 265, 268, 271, 274, 277, 281, 282, 286, 287, 294, 305, 311, 315, 319, 328, 343, 351, 364, 365, 384, 385, 395, 410, 422, 423, 429, 430, 436, 437, 443, 444, 449, 458, 459, 470, 483, 484, 485, 493, 494, 495, 503, 504, 505, 513, 514, 520, 528, 529, 534, 542, 543, 544, 552, 553, 554, 562, 563, 586, 623, 624, 628, 629, 633, 634, 640, 653, 654, 659, 680, 681, 693, 694, 706, 707, 719, 720, 732, 733, 751, 752, 764, 765, 777, 778, 790, 791, 803, 804, 817, 818, 828, 841, 842, 848, 853, 854, 875, 876, 885, 897, 904, 905, 917, 918, 924, 955, 956, 973, 974, 983, 984, 993, 994, 1003, 1004, 1013, 1014, 1023, 1024, 1033, 1034, 1043, 1044, 1052, 1054, 1069, 1070, 1076, 1077, 1078, 1079, 1083, 1084, 1090, 1094, 1095, 1097, 1111, 1112, 1117, 1119 | Compatibility exports, read/write convenience APIs, metadata helpers, date/string/buffer conversions |
| `safe/src/memory.rs` | 23, 29, 31, 34, 40, 42, 46, 63, 65, 71, 75, 83, 95, 97, 131, 132, 135, 136, 139, 141, 147, 149, 155, 164, 181, 182, 186, 199, 210, 211, 234, 236, 242, 253, 255, 261, 265, 266, 270, 275, 276, 280, 285, 286, 290, 295, 296, 300, 305, 306, 307, 312, 313, 314, 319, 320, 335, 336, 343, 344, 350, 362, 363, 372, 379, 380, 386, 398, 399, 408, 415, 416, 417, 426, 427, 436, 439, 440, 445, 446, 452, 453, 454, 458, 459, 464, 465, 480, 481, 517, 521, 522, 531, 536, 538, 542, 545, 546, 551, 554, 555, 561, 562, 570, 575, 577, 581, 584, 585 | Allocator callbacks, default libc allocation, opaque handle lifetime, create/destroy functions, create-time longjmp trap |
| `safe/src/io.rs` | 5, 13, 16, 18, 22, 30, 33, 35, 39, 47, 53, 61, 67, 68, 75, 76, 93, 94, 109, 110, 126, 127, 138, 139, 150, 151, 172, 173, 180, 181, 194, 195, 202, 203, 214, 215, 226, 227, 242, 243, 250, 251, 255, 256, 317, 318, 319, 322, 323, 324, 327, 328, 333, 336, 337, 343, 346, 347, 351, 354, 355, 359, 362, 363, 370, 373, 374, 375, 378, 379, 384, 387, 388, 389, 392, 393, 397, 400, 401, 405, 408, 409, 415, 425, 426, 429, 432, 433, 434, 437, 438, 439 | stdio callbacks, user callback registration, user-transform and progressive callback pointers, bridge wrappers |
| `safe/src/error.rs` | 31, 34, 38, 41, 45, 46, 49, 50, 54, 59, 60, 64, 70, 71, 75, 86, 87, 91, 96, 97, 101, 107, 108, 112, 117, 118, 133, 134, 141, 142, 147, 180, 181, 182, 193, 195, 203, 204, 208, 211, 212, 216, 219, 220, 224, 227, 228, 232, 235, 236, 240, 243, 244, 248, 251, 252, 258, 261, 262, 263, 266, 267, 271 | Error/warning callbacks, panic-to-error path, `png_longjmp`, longjmp buffer release, bridge wrappers |
| `safe/src/write_util.rs` | 5, 6, 10, 14, 15, 16, 21, 22, 28, 33, 34, 39, 44, 45, 50, 55, 56, 57, 62, 63, 64, 72, 73, 74, 82, 83, 87, 95, 96, 100, 108, 109, 113, 121, 122, 126, 134, 135, 139, 147, 148, 152, 160, 161, 165, 173, 174, 178, 186, 187, 191 | Public write utility exports and bridge dispatch |
| `safe/src/write.rs` | 19, 21, 26, 28, 33, 35, 40, 42, 47, 49, 54, 56, 61, 68, 73, 75, 80, 81, 85, 90, 91, 92, 97, 98, 99, 104, 105, 110, 115, 116, 117, 122, 123, 124, 129, 130, 136, 141, 142, 143, 151, 152, 153 | Public write exports and dispatch into write runtime/bridge helpers |
| `safe/src/simplified_runtime.rs` | 173, 178, 180, 181, 192, 204, 226, 250, 260, 264, 267, 271, 278, 695, 721, 1246, 1301, 1698, 2264, 2265, 2284, 2285, 2407, 2502, 2520, 2543, 2566, 2578, 2591, 2602, 2607, 2636, 2644, 2657, 2668, 2680, 2693, 2704, 2713, 2727, 2747, 2748 | Simplified image opaque state, file/stdin/stdout reads/writes, buffer/stride pointer handling |
| `safe/src/read_progressive.rs` | 11, 24, 25, 45, 191, 201, 211, 219, 226, 236, 249, 263, 269, 271, 272, 281, 283, 291, 295, 296, 317, 327, 335, 358, 371, 388, 396, 403, 404, 414, 438, 439, 443, 448, 449, 450 | Progressive read bridge calls, suspend/resume state, progressive callbacks |
| `safe/src/set.rs` | 9, 202, 203, 208, 221, 222, 223, 228, 229, 234, 408, 409, 414, 423, 424, 428, 436, 437, 441, 449, 450, 462, 470, 489, 502, 503, 504, 512, 513, 525, 526, 531 | Public setter exports, metadata pointer transfer, unknown chunk control |
| `safe/src/colorspace.rs` | 58, 68, 220, 237, 262, 275, 303, 332, 345, 377, 407, 440, 441, 452, 453, 466, 467, 485, 486, 505, 506, 516, 517, 525, 526, 550, 551, 601, 602, 622, 623, 663 | Colorspace transform exports, C pointer writes for cHRM/XYZ getters, error callbacks |
| `safe/src/read_transform.rs` | 22, 40, 50, 71, 72, 78, 79, 85, 86, 92, 93, 99, 100, 106, 107, 113, 114, 120, 121, 134, 154, 155, 163, 168, 169, 175, 176, 182, 183, 189, 190 | Read transform exports and internal quantize bridge |
| `safe/src/common.rs` | 128, 201, 212, 213, 240, 241, 245, 246, 250, 251, 255, 256, 260, 261, 265, 266, 291, 304, 314, 315, 321, 322, 332, 333, 369, 382, 383, 401, 402, 406, 407 | ABI guards, common exported helpers, `memcmp`, `gmtime_r`, output pointer writes |
| `safe/src/simplified.rs` | 43, 60, 63, 74, 131, 203, 280, 281, 290, 294, 295, 304, 308, 309, 314, 323, 324, 336, 340, 341, 342, 345, 346, 354, 370, 371, 379, 388, 389, 398 | Public simplified ABI, image pointer dereference, bridge dispatch |
| `safe/src/get.rs` | 6, 216, 217, 218, 429, 430, 440, 445, 446, 450, 454, 455, 459, 463, 464, 468, 472, 473, 477, 481, 482 | Getter bridge extern declarations, macro-generated getters, output pointer handling |
| `safe/src/write_transform.rs` | 8, 9, 14, 20, 21, 28, 40, 41, 48, 60, 61, 66, 72, 73, 78, 84, 85, 86, 92, 93, 94, 100, 101, 102 | Public write transform exports and bridge dispatch |
| `safe/src/write_runtime.rs` | 143, 773, 1246, 1257, 1263, 1279, 1289, 1407, 1426, 1435, 1523, 1527, 1533, 1539, 1540, 1544, 1550, 1597, 1615 | Write runtime row pointer reads, callback calls, output buffer and chunk write handoff |
| `safe/src/types.rs` | 251, 252, 253, 254, 255, 256, 257, 259, 261, 263, 264, 265, 266 | C function-pointer type aliases and ABI callback signatures |
| `safe/src/chunks.rs` | 31, 34, 39, 42, 44, 48, 49, 52, 58, 149 | Central chunk warning/error/app-callback dispatch and read core mutation |
| `safe/src/zlib.rs` | 23, 24, 31, 114, 116, 117, 139, 152, 171 | zlib stream function pointers and direct zlib inflate calls |
| `safe/src/interlace.rs` | 31, 82, 83 | Interlace exported setter and error callback |
| `safe/src/state.rs` | 75, 227 | `unsafe impl Send` for `PngStructState` and `PngInfoState` registry values |
| `safe/src/read_util.rs` | 145, 146 | Unsafe byte copy used by read utility helpers |

## 3. Remaining unsafe FFI beyond the original ABI/API boundary

Runtime link evidence from:

```bash
readelf -d safe/target/release/abi-stage/usr/lib/x86_64-linux-gnu/libpng16.so.16.43.0
```

shows SONAME `libpng16.so.16` and `NEEDED` entries for `libz.so.1`,
`libm.so.6`, `libgcc_s.so.1`, `libc.so.6`, and
`ld-linux-x86-64.so.2`. `safe/build.rs` and
`safe/tools/stage-install-tree.sh` pass `-lz -lm -ldl -lpthread -lrt -lutil
-lgcc_s` while linking the staged shared object; after linker
`--as-needed` behavior, `libz`, `libm`, `libgcc_s`, `libc`, and the dynamic
loader remain in the dynamic section, while `dl`, `pthread`, `rt`, and `util`
are not retained as `NEEDED` libraries in the current staged object.

| FFI surface | Evidence | Why it remains | Safer replacement path |
| --- | --- | --- | --- |
| zlib inflate ABI | `safe/src/zlib.rs:31` declares `zlibVersion`, `inflateInit_`, `inflate`, `inflateEnd`; call sites at `safe/src/zlib.rs:116-171` | libpng-compatible handling of compressed ancillary chunks still uses platform zlib. | Replace direct zlib inflate FFI with a pure Rust or `flate2` inflate path if compatibility, memory-limit semantics, and performance are acceptable. |
| libc allocation and longjmp | `safe/src/memory.rs:23` declares `_setjmp` and `longjmp`; `safe/src/memory.rs:40`, `safe/src/memory.rs:42`, `safe/src/memory.rs:97`, `safe/src/memory.rs:142`, `safe/src/memory.rs:150` use `calloc`, `malloc`, and `free`; `safe/src/error.rs:181` implements `png_longjmp` | The libpng ABI exposes default C allocation behavior, user allocator callbacks, and longjmp-based error control. | Keep default allocation behind a narrower allocator abstraction; reduce or remove `setjmp`/`longjmp` handling only if libpng ABI compatibility is relaxed, since callers rely on that model. |
| stdio/file helpers | `safe/src/io.rs:16`, `safe/src/io.rs:33`, `safe/src/io.rs:48`; `safe/src/bridge_ffi.rs:741-776`; `safe/src/simplified_runtime.rs:260-278`, `safe/src/simplified_runtime.rs:2704` | `png_init_io` and simplified file/stdout APIs accept or expose C `FILE *` behavior. | Replace libc stdio/file helpers in simplified APIs with Rust-owned file reads/writes where the libpng ABI permits ownership and callback semantics to remain compatible. |
| String, time, and comparison helpers | `safe/src/bridge_ffi.rs:3966`, `safe/src/bridge_ffi.rs:4005`, `safe/src/bridge_ffi.rs:4019`, `safe/src/bridge_ffi.rs:4028`, `safe/src/bridge_ffi.rs:4043`, `safe/src/bridge_ffi.rs:4050`, `safe/src/bridge_ffi.rs:4209`, `safe/src/bridge_ffi.rs:4210` use `strlen`; `safe/src/bridge_ffi.rs:3497`, `safe/src/bridge_ffi.rs:3502`, `safe/src/bridge_ffi.rs:3527`, `safe/src/bridge_ffi.rs:3532` use `atof`; `safe/src/common.rs:232` uses `memcmp`; `safe/src/common.rs:414` uses `gmtime_r` | The port preserves C string, numeric parsing, byte comparison, and RFC1123 time formatting semantics for ABI compatibility. | Move parsing and formatting to Rust wrappers that validate C strings at the boundary, then operate on Rust slices/strings internally. |
| Application callbacks | `safe/src/error.rs:31`, `safe/src/io.rs:5`, `safe/src/io.rs:22`, `safe/src/io.rs:39`, `safe/src/chunks.rs:31`, `safe/src/read_progressive.rs:191`, `safe/src/read_progressive.rs:211`, `safe/src/read_progressive.rs:226`, `safe/src/write_runtime.rs:1523`, `safe/src/write_runtime.rs:1527` | Existing libpng callers register raw callback pointers and `void *` user data. | Replace raw callback registration/calls internally with safer wrapper types that validate callback pointers, lifetimes, and user data at the ABI boundary. |
| Internal Rust-to-Rust extern shims | `safe/src/get.rs:6`, `safe/src/set.rs:9`, `safe/src/read.rs:38`, `safe/src/read_progressive.rs:11`, `safe/src/read_transform.rs:22`, and bridge definitions in `safe/src/bridge_ffi.rs` | Some earlier bridge seams remain as `extern "C"` calls even when both caller and callee are Rust. | Collapse Rust-to-Rust internal `extern "C"` shims back to ordinary Rust calls where they are not needed for exported ABI or C bridge interop. |
| Registry `Send` assertions | `safe/src/state.rs:75`, `safe/src/state.rs:227` | Global synchronized registries store raw pointer-valued ABI handles and callback pointers. | Revisit `unsafe impl Send` if a more constrained synchronization or ownership model can preserve ABI behavior. |

## 4. Remaining issues

Validation evidence is pinned to existing artifacts and was not regenerated or
retargeted. `git -C validator rev-parse HEAD`,
`validator-case-inventory.json`, and `validator-report.md` all name validator
commit `cc99047419226144eec3c1ab87873052bd9abedc`. The final local-override
validator result in `validator/artifacts/libpng-safe-final/results/libpng/summary.json`
is 105/105 cases passed, 0 failed, with no validator bug exceptions.
`validator-report.md` records 5/5 source cases and 100/100 usage cases, all
run in validator `original` mode with local safe `.deb` overrides from
`validator-overrides/libpng/`.

Packaging caveat: `validator-case-inventory.json` records
`proof_rejects_original_override: true` for this validator commit.
`validator-report.md` explains that proof/site generation rejects original-mode
result JSON when `override_debs_installed` is true, so proof and site targets
were intentionally not run for the final local-override validation. This is a
proof-tooling limitation, not a validator bug exception, and was not used to
skip any libpng matrix check.

Local build/test harnesses present in the tree include:
`safe/tools/check-exports.sh`, `safe/tools/check-headers.sh`,
`safe/tools/check-link-compat.sh`, `safe/tools/check-install-surface.sh`,
`safe/tools/check-build-layout.sh`, `safe/tools/check-core-smoke.sh`,
`safe/tools/check-read-core.sh`, `safe/tools/check-read-transforms.sh`,
`safe/tools/run-read-tests.sh`, `safe/tools/run-write-tests.sh`,
`safe/tools/run-upstream-tests.sh`, `safe/tools/check-examples-and-tools.sh`,
`safe/tools/run-cve-regressions.sh`, `safe/tools/run-dependent-regressions.sh`,
and `safe/tools/check-package-artifacts.sh`. Test drivers live under
`safe/tests/core-smoke`, `safe/tests/read-core`,
`safe/tests/read-transforms`, `safe/tests/upstream`,
`safe/tests/cve-regressions`, and `safe/tests/dependents`.

Remaining caveats and coverage:

| Topic | Current evidence | Remaining gap |
| --- | --- | --- |
| Performance versus `original/` | `safe/tools/check-link-compat.sh:99-114`, `safe/tests/upstream/timepng.sh`, `safe/tools/run-upstream-tests.sh:57-66`, and `safe/tools/check-examples-and-tools.sh:13-36` build or run `timepng` as smoke/consumer coverage. | There is no comprehensive quantitative benchmark artifact comparing safe Rust throughput or latency against original C libpng across read/write/transforms. Treat performance equivalence as unproven beyond smoke coverage. |
| Bit-for-bit / byte equivalence | Exact comparisons exist for selected areas: `safe/tests/upstream/common.sh:448` uses `cmp -s` for `png-fix-itxt` identity on `pngtest.png`; `safe/tools/check-headers.sh:53` and `safe/tools/check-headers.sh:63` compare staged headers to baselines; `safe/tools/check-package-artifacts.sh:575` compares packaged examples; `safe/tests/read-transforms/simplified_read_driver.c:116`, `safe/tests/read-transforms/simplified_read_driver.c:202`, and `safe/tests/read-transforms/simplified_read_driver.c:212` compare simplified read buffers; `safe/tests/read-transforms/update_info_driver.c:458` and `safe/tests/read-transforms/update_info_driver.c:468` compare interlace/update-info rows; `safe/tests/dependents/write_packing_indices.c:127` compares exact packed palette indices; `safe/tools/run-write-tests.sh:62-86` runs the upstream `pngstest-*` wrapper matrix listed in `validator-report.md:30-38`; install and export baselines are compared by `safe/tools/check-build-layout.sh`, `safe/tools/check-install-surface.sh`, and `safe/tools/check-exports.sh`. | Existing artifacts do not prove exhaustive byte-for-byte equivalence for every libpng read/write behavior, transform combination, compression choice, metadata path, and application callback pattern. Covered exact-comparison areas are the listed headers/install/export/package/examples, selected read-transform buffers, selected simplified-read stride cases, selected dependent write/readback behavior, and upstream `pngstest-*` wrappers. |
| CVE coverage | `relevant_cves.json` reviewed 70 CVEs and selected 12 relevant non-memory-safety or Rust-surviving issues: CVE-2004-0599, CVE-2007-2445, CVE-2007-5268, CVE-2009-2042, CVE-2010-0205, CVE-2011-3328, CVE-2014-0333, CVE-2017-12652, CVE-2018-13785, CVE-2004-0598, CVE-2013-6954, CVE-2016-10087. `safe/tests/cve-regressions/coverage.json` maps all 12 selected CVEs to tests or invariants. `safe/tools/run-cve-regressions.sh` validates that each selected CVE has coverage entries and that each referenced path exists before running read/write regression drivers. | The selected CVE set intentionally excludes primary memory-corruption CVEs and utility-only issues unless they inform parser/state invariants. Coverage is explicit for the 12 selected CVEs, not an exhaustive proof for all historical libpng CVEs. |
| Dependent coverage | `dependents.json` inventories GIMP, LibreOffice Draw, Scribus, WebKitGTK, GDK Pixbuf, Cairo, SDL2_image, feh, Netpbm, XSane, the R `png` package, and pngquant. Actual validator usage execution covers Netpbm and pngquant usage cases: `validator-case-inventory.json` lists 100 usage cases, all `usage-netpbm-*` or `usage-pngquant-*`, and `validator-report.md` records 100/100 usage cases passed. `safe/tests/dependents/` contains targeted C regressions for API patterns: palette expansion/shift, `png_set_sig_bytes` with custom error handling, and `png_set_packing` write/readback. | GIMP, LibreOffice Draw, Scribus, WebKitGTK, GDK Pixbuf, Cairo, SDL2_image, feh, XSane, and the R `png` package are dependent inventory, not direct end-to-end validation in the current artifacts unless a future artifact proves direct coverage. |
| TODO/FIXME/XXX search | The TODO/FIXME/XXX search over `safe/src`, `safe/tests`, `safe/tools`, and `safe/debian`, excluding generated Debian package-output directories, found no live Rust-port TODO/FIXME markers in `safe/src`. Matches are upstream/debian prose and patch history, Debian docs/manifests, `safe/debian/rules` copying upstream `TODO`, and `XXX` in `mktemp` templates such as `safe/tools/refresh-source-snapshot.sh:11` and `safe/tests/dependents/write_packing_indices.c:143`. `safe/TODO` is upstream libpng TODO content, not automatically Rust-port-specific defects. | No live Rust-port TODO marker was identified by this search, but the absence of markers is not a substitute for the validation caveats above. |

## 5. Dependencies and other libraries used

Direct Rust dependencies from `safe/Cargo.toml` and resolved versions from
`cargo tree --locked -e normal,build --manifest-path safe/Cargo.toml`:

| Dependency | Manifest requirement | Resolved version | Use in this port | Unsafe posture from manual inspection |
| --- | --- | --- | --- | --- |
| `flate2` | `1` | `1.1.9` | zlib/deflate encoding support in the write runtime | No `#![forbid(unsafe_code)]` found under `safe/debian/cargo-home/registry/src/index.crates.io-1949cf8c6b5b557f/flate2-1.1.9/src`; manual text search found 37 `unsafe` matches, including FFI/miniz stream handling. |
| `libc` | `0.2` | `0.2.183` | C ABI types and direct libc calls | No `#![forbid(unsafe_code)]`; manual text search found 429 `unsafe` matches across multi-platform libc bindings. This is expected for the libc crate. |
| `png` | `0.18.1` | `0.18.1` | Safe Rust PNG encoder/decoder building blocks for simplified and write paths | `safe/debian/cargo-home/registry/src/index.crates.io-1949cf8c6b5b557f/png-0.18.1/src/lib.rs:62` declares `#![forbid(unsafe_code)]`; manual text search found 0 `unsafe` matches in `src`. |
| `cc` | `1.2` | `1.2.58` | Compiler discovery for staged shared-library linking in `build.rs` | No `#![forbid(unsafe_code)]`; manual text search found 19 `unsafe` matches, mostly compiler-support internals and jobserver/fd helpers. Build dependency only. |

Transitive dependencies are supporting context: `flate2` pulls in
`crc32fast 1.5.0` and `miniz_oxide 0.8.9`; `png` pulls in `bitflags 2.11.0`,
`crc32fast 1.5.0`, `fdeflate 0.3.7`, `flate2 1.1.9`, and
`miniz_oxide 0.8.9`; `cc` pulls in `find-msvc-tools 0.1.9` and
`shlex 1.3.0`. Manual registry inspection found `#![forbid(unsafe_code)]`
in `png`, `fdeflate`, `miniz_oxide`, and `adler2`; text matches for `unsafe`
remain in some transitive crates such as `crc32fast`, `simd-adler32`,
`bitflags`, `find-msvc-tools`, and `shlex`. These are text-match counts, not
`cargo geiger` item counts.

System/runtime libraries and tools used by the current build and validation
surface include zlib (`libz.so.1`), libm (`libm.so.6`), libc (`libc.so.6`),
`libgcc_s.so.1`, the dynamic loader (`ld-linux-x86-64.so.2`), link arguments
for `dl`, `pthread`, `rt`, and `util`, Cargo, rustc, gcc/cc, dpkg-dev,
debhelper, mawk, python3, `pkg-config`, `nm`, `objdump`, `readelf`, and
optional `cargo geiger`.

`cbindgen` and `bindgen` are not present in `safe/Cargo.toml`,
`safe/Cargo.lock`, or `safe/build.rs` (`rg -n "cbindgen|bindgen"
safe/Cargo.toml safe/build.rs safe/Cargo.lock` returned no matches).

Manual dependency unsafe inspection command used because `cargo geiger` was
not installed:

```bash
registry_root="safe/debian/cargo-home/registry/src/index.crates.io-1949cf8c6b5b557f"
for crate in flate2-1.1.9 libc-0.2.183 png-0.18.1 cc-1.2.58 \
  crc32fast-1.5.0 miniz_oxide-0.8.9 fdeflate-0.3.7 simd-adler32-0.3.9 \
  adler2-2.0.1 bitflags-2.11.0 cfg-if-1.0.4 find-msvc-tools-0.1.9 shlex-1.3.0
do
  dir="$registry_root/$crate"
  rg -n '#!\[forbid\(unsafe_code\)\]' "$dir"/src "$dir"/Cargo.toml 2>/dev/null
  rg -n '\bunsafe\b|unsafe extern "C"|#\[unsafe|unsafe impl' "$dir"/src 2>/dev/null | wc -l
done
```

## 6. How this document was produced

This document was produced from the checked-out repository, prepared validator
evidence, CVE/dependent inventories, and existing build/package artifacts. No
remote clone, pull, refetch, validator retargeting, CVE rediscovery, dependent
rediscovery, package rebuild from scratch, or validator inventory regeneration
was performed for the documentation pass.

Initial workspace status from `git status --short` showed only existing
untracked Debian build/package outputs under `safe/debian/`, including
`safe/debian/.debhelper/`, `safe/debian/build-tools/`,
`safe/debian/cargo-home/`, `safe/debian/tmp/`,
package-output directories for `libpng-dev`, `libpng-tools`,
`libpng16-16-udeb`, `libpng16-16t64`, and
`safe/debian/upstream-source-root/`. They were treated as existing artifacts,
not source edits.

Refreshable commands and evidence used:

```bash
git status --short
test -f safe/PORT.md
cargo metadata --locked --format-version 1 --no-deps --manifest-path safe/Cargo.toml
cargo tree --locked -e normal,build --manifest-path safe/Cargo.toml
wc -l safe/abi/exports.txt
sed -n '1,140p' safe/tools/check-exports.sh
nl -ba safe/src/common.rs | sed -n '100,155p'
rg -n 'OnceLock|HashMap|PngStructState|PngInfoState|register_png|register_info|remove_png|remove_info' safe/src/state.rs
rg -n 'zlibVersion|inflateInit_|inflate\(|inflateEnd|_setjmp|longjmp|malloc|calloc|free|fopen|fclose|ftell|fseek|fread|fwrite|fflush|strlen|atof|memcmp|gmtime_r' safe/src/memory.rs safe/src/error.rs safe/src/io.rs safe/src/bridge_ffi.rs safe/src/simplified_runtime.rs safe/src/zlib.rs safe/src/common.rs
readelf -d safe/target/release/abi-stage/usr/lib/x86_64-linux-gnu/libpng16.so.16.43.0
git ls-files safe/src safe/build.rs safe/cshim safe/tools/build_support/build_support.rs | xargs rg -n '\bunsafe\b|unsafe extern "C"|#\[unsafe|unsafe impl' >/tmp/libpng-safe-unsafe-tracked.txt
grep -RIn '\bunsafe\b' safe >/tmp/libpng-safe-unsafe-full.txt
rg -n 'TODO|FIXME|XXX' safe/src safe/tests safe/tools safe/debian
rg -n 'timepng|performance|bench|perf' safe/tools safe/tests validator-report.md original safe/TODO
rg -n 'cmp -s|memcmp|exact|bit-for-bit|byte-for-byte|pngstest|baseline' safe/tools safe/tests validator-report.md
jq -c '.' relevant_cves.json
jq -c '.' safe/tests/cve-regressions/coverage.json
jq -c '.' dependents.json
git -C validator rev-parse HEAD
jq -r '.validator_commit' validator-case-inventory.json
jq -c '{cases,passed,failed}' validator/artifacts/libpng-safe-final/results/libpng/summary.json
rg -n 'cc99047419226144eec3c1ab87873052bd9abedc|105/105|validator bug|override|passed|failed' validator-report.md validator-case-inventory.json
cargo geiger --manifest-path safe/Cargo.toml || true
rg -n 'cbindgen|bindgen' safe/Cargo.toml safe/build.rs safe/Cargo.lock
```

`cargo geiger` was unavailable in this environment (`error: no such command:
geiger`), so dependency unsafe posture was documented from the manual registry
inspection command shown in section 5, preferring the existing
`safe/debian/cargo-home/registry/src` sources.

Post-edit verification for this document should include the implementation
commands listed by the phase source:

```bash
cargo build --locked --release --manifest-path safe/Cargo.toml
safe/tools/stage-install-tree.sh
safe/tools/check-exports.sh
safe/tools/check-headers.sh
safe/tools/check-link-compat.sh
safe/tools/check-install-surface.sh
safe/tools/check-build-layout.sh
safe/tools/check-core-smoke.sh
safe/tools/check-read-core.sh
safe/tools/check-read-transforms.sh
safe/tools/run-read-tests.sh
safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-large-stride pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha
safe/tools/run-upstream-tests.sh
safe/tools/check-examples-and-tools.sh
safe/tools/run-cve-regressions.sh --mode all
safe/tools/run-dependent-regressions.sh
jq -e '.passed == .cases and .failed == 0' validator/artifacts/libpng-safe-final/results/libpng/summary.json
validator_commit="$(git -C validator rev-parse HEAD)"
test "$validator_commit" = "cc99047419226144eec3c1ab87873052bd9abedc"
test "$validator_commit" = "$(jq -r '.validator_commit' validator-case-inventory.json)"
git diff --check -- safe/PORT.md
```
