; ModuleID = bmb_bootstrap
declare void @println(i64) nofree nounwind willreturn
declare void @print_str(ptr nocapture readonly) nofree nounwind willreturn
declare void @println_str(ptr nocapture readonly) nofree nounwind willreturn
declare void @eprint(i64) nofree nounwind willreturn
declare void @eprintln(i64) nofree nounwind willreturn
declare void @eprint_str(ptr nocapture readonly) nofree nounwind willreturn
declare void @eprintln_str(ptr nocapture readonly) nofree nounwind willreturn
declare void @eprint_f64(double) nofree nounwind willreturn
declare void @eprintln_f64(double) nofree nounwind willreturn
declare i64 @llvm.abs.i64(i64, i1)
declare i64 @llvm.smin.i64(i64, i64)
declare i64 @llvm.smax.i64(i64, i64)
declare i64 @bmb_pow(i64, i64) nofree nosync nounwind willreturn memory(none) speculatable
declare noalias ptr @bmb_array_push(ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_array_pop(ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_array_concat(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_array_slice(ptr nocapture readonly, i64, i64) nofree nosync nounwind willreturn
declare i64 @bmb_array_len(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare noalias ptr @bmb_string_new(ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_from_cstr(ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @bmb_string_len(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare i64 @bmb_string_char_at(ptr nocapture readonly, i64) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare noalias ptr @bmb_string_slice(ptr nocapture readonly, i64, i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_concat(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_concat3(ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_concat5(ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_concat7(ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @bmb_string_eq(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @bmb_string_cmp(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @bmb_string_hash(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare i64 @bmb_string_starts_with(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare i64 @bmb_string_ends_with(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare i64 @bmb_string_contains(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare i64 @bmb_string_index_of(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare noalias ptr @bmb_string_trim(ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_replace(ptr nocapture readonly, ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_to_upper(ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_to_lower(ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_repeat(ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare i64 @bmb_string_is_empty(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read) speculatable
declare i64 @bmb_string_count(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare noalias ptr @bmb_string_reverse(ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_pad_left(ptr nocapture readonly, i64, i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_pad_right(ptr nocapture readonly, i64, i64) nofree nosync nounwind willreturn
declare i64 @bmb_string_last_index_of(ptr nocapture readonly, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare noalias ptr @bmb_chr(i64) nofree nosync nounwind willreturn
declare i64 @bmb_ord(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare noalias ptr @bmb_int_to_string(i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_f64_to_string(double) nofree nosync nounwind willreturn
declare noalias ptr @bmb_to_hex(i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_to_binary(i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_to_octal(i64) nofree nosync nounwind willreturn
declare i64 @bmb_parse_int(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare double @bmb_parse_f64(ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare noalias ptr @bmb_fast_i2s(i64) nofree nosync nounwind willreturn
declare i64 @bmb_file_exists(ptr nocapture readonly) nofree nounwind willreturn
declare i64 @bmb_file_size(ptr nocapture readonly) nofree nounwind willreturn
declare i64 @is_dir(ptr nocapture readonly) nofree nounwind willreturn
declare noalias ptr @list_dir(ptr nocapture readonly) nofree nounwind willreturn
declare i64 @remove_file(ptr nocapture readonly) nofree nounwind willreturn
declare i64 @llvm.sadd.sat.i64(i64, i64)
declare i64 @llvm.ssub.sat.i64(i64, i64)
declare {i64, i1} @llvm.smul.with.overflow.i64(i64, i64)
declare noalias ptr @bmb_read_file(ptr nocapture readonly) nofree nounwind willreturn
declare i64 @bmb_write_file(ptr nocapture readonly, ptr nocapture readonly) nofree nounwind willreturn
declare i64 @bmb_append_file(ptr nocapture readonly, ptr nocapture readonly) nofree nounwind willreturn
declare i64 @write_file_newlines(ptr nocapture readonly, ptr nocapture readonly) nofree nounwind willreturn
declare i64 @bmb_sb_new() nofree nosync nounwind willreturn
declare i64 @bmb_sb_push(i64, ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @sb_push_range(i64, ptr nocapture readonly, i64, i64) nofree nosync nounwind willreturn
declare i64 @bmb_sb_push_int(i64, i64) nofree nosync nounwind willreturn
declare i64 @bmb_sb_push_char(i64, i64) nofree nosync nounwind willreturn
declare i64 @bmb_sb_push_escaped(i64, ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @bmb_sb_len(i64) nofree nosync nounwind willreturn memory(read)
declare noalias ptr @bmb_sb_build(i64) nofree nosync nounwind willreturn
declare i64 @bmb_sb_clear(i64) nofree nosync nounwind willreturn
declare i64 @bmb_sb_contains(i64, ptr nocapture readonly) nofree nosync nounwind willreturn memory(read)
declare i64 @bmb_system(ptr nocapture readonly)
declare void @bmb_exit(i64) noreturn nounwind
declare i64 @bmb_time_ms() nofree nounwind willreturn
declare void @bmb_sleep_ms(i64) nofree nounwind willreturn
declare i64 @bmb_random_i64() nofree nosync nounwind willreturn
declare void @bmb_random_seed(i64) nofree nosync nounwind willreturn
declare noalias ptr @bmb_getenv(ptr nocapture readonly) nofree nosync nounwind willreturn
declare noalias ptr @bmb_system_capture(ptr nocapture readonly)
declare noalias ptr @bmb_read_line() nofree nounwind willreturn
declare i64 @bmb_string_free(ptr) nosync nounwind willreturn
declare i64 @bmb_sb_free(i64) nosync nounwind willreturn
declare i64 @bmb_arena_mode(i64) nofree nosync nounwind willreturn
declare i64 @bmb_arena_reset() nofree nosync nounwind willreturn
declare i64 @bmb_arena_save() nofree nosync nounwind willreturn
declare i64 @bmb_arena_restore() nofree nosync nounwind willreturn
declare i64 @bmb_arena_usage() nofree nosync nounwind willreturn
declare i64 @arg_count() nofree nosync nounwind willreturn
declare noalias ptr @get_arg(i64) nofree nosync nounwind willreturn
declare void @bmb_panic(ptr nocapture readonly) noreturn nounwind
declare noalias ptr @malloc(i64) nofree nosync nounwind willreturn
declare noalias ptr @calloc(i64, i64) nofree nosync nounwind willreturn
declare void @free(ptr) nosync nounwind willreturn
declare double @llvm.sin.f64(double)
declare double @llvm.cos.f64(double)
declare double @llvm.tan.f64(double)
declare double @llvm.atan.f64(double)
declare double @llvm.atan2.f64(double, double)
declare double @llvm.log.f64(double)
declare double @llvm.log2.f64(double)
declare double @llvm.log10.f64(double)
declare double @llvm.exp.f64(double)
declare double @llvm.pow.f64(double, double)
declare void @print_f64(double) nofree nounwind willreturn
declare void @println_f64(double) nofree nounwind willreturn
declare void @bmb_print_i64(i64) nofree nounwind willreturn
declare void @puts_cstr(ptr nocapture readonly) nofree nounwind willreturn
declare void @store_u8(ptr, i64, i64) nofree nosync nounwind willreturn memory(argmem: write)
declare i64 @load_u8(ptr, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare void @store_i64(ptr, i64, i64) nofree nosync nounwind willreturn memory(argmem: write)
declare i64 @load_i64(ptr, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @char_at(ptr nocapture readonly, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare noalias ptr @hashmap_new() nofree nosync nounwind willreturn
declare void @hashmap_insert(ptr, ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare i64 @hashmap_get(ptr, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare void @hashmap_free(ptr) nosync nounwind willreturn
declare i64 @hashmap_remove(ptr, ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @hashmap_contains(ptr, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @hashmap_len(ptr) nofree nosync nounwind willreturn memory(argmem: read)
declare noalias ptr @str_hashmap_new() nofree nosync nounwind willreturn
declare i64 @str_hashmap_insert(ptr, ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare i64 @str_hashmap_get(ptr, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare void @str_hashmap_free(ptr) nosync nounwind willreturn
declare i64 @str_hashmap_contains(ptr, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @str_hashmap_len(ptr) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @str_hashmap_remove(ptr, ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @str_hashmap_keys(ptr) nofree nosync nounwind willreturn
declare i64 @str_hashmap_values(ptr) nofree nosync nounwind willreturn
declare noalias ptr @reg_cached_lookup(ptr, ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare noalias ptr @vec_new() nofree nosync nounwind willreturn
declare i64 @vec_push(ptr, i64) nofree nosync nounwind willreturn
declare i64 @vec_get(ptr, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare void @vec_set(ptr, i64, i64) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare i64 @vec_len(ptr) nofree nosync nounwind willreturn memory(argmem: read)
declare void @vec_free(ptr) nosync nounwind willreturn
declare void @vec_reverse(ptr) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare i64 @vec_contains(ptr, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @vec_index_of(ptr, i64) nofree nosync nounwind willreturn memory(argmem: read)
declare void @vec_swap(ptr, i64, i64) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @vec_sort(ptr) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare noalias ptr @vec_with_capacity(i64) nofree nosync nounwind willreturn
declare void @vec_clear(ptr) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare i64 @sb_println(i64) nofree nounwind willreturn
declare i64 @sb_with_capacity(i64) nofree nosync nounwind willreturn
declare double @llvm.fabs.f64(double)
declare double @llvm.floor.f64(double)
declare double @llvm.ceil.f64(double)
declare double @llvm.round.f64(double)
declare double @llvm.sqrt.f64(double)
declare double @llvm.minnum.f64(double, double)
declare double @llvm.maxnum.f64(double, double)
declare i64 @bmb_f64_is_nan(double) nofree nosync nounwind willreturn memory(none) speculatable
declare i64 @bmb_f64_to_int(double) nofree nosync nounwind willreturn memory(none) speculatable
declare i64 @bmb_delete_file(ptr) nofree nounwind willreturn
declare noalias ptr @bmb_getcwd() nofree nosync nounwind willreturn
declare i64 @vec_pop(ptr) nofree nosync nounwind willreturn
declare i64 @vec_cap(ptr) nofree nosync nounwind willreturn
declare i64 @bmb_time_ns() nofree nounwind willreturn
declare noalias ptr @bmb_exec_output(ptr, ptr) nofree nounwind willreturn
declare i64 @bmb_string_split(ptr, ptr) nofree nosync nounwind willreturn
declare noalias ptr @bmb_string_join(i64, ptr) nofree nosync nounwind willreturn
declare i64 @bmb_sb_push_f64(i64, double) nofree nosync nounwind willreturn
declare i64 @bmb_sb_push_hex(i64, i64) nofree nosync nounwind willreturn
declare i64 @bmb_sb_push_bool(i64, i64) nofree nosync nounwind willreturn
declare i64 @llvm.ctpop.i64(i64)
declare i64 @llvm.ctlz.i64(i64, i1)
declare i64 @llvm.cttz.i64(i64, i1)
declare i64 @llvm.bitreverse.i64(i64)
declare i64 @llvm.bswap.i64(i64)
declare i64 @llvm.fshl.i64(i64, i64, i64)
declare i64 @llvm.fshr.i64(i64, i64, i64)
declare i64 @vec_slice(ptr, i64, i64) nofree nosync nounwind willreturn
declare void @vec_extend(ptr, ptr) nofree nosync nounwind willreturn
declare i64 @vec_remove(ptr, i64) nofree nosync nounwind willreturn
declare void @vec_insert(ptr, i64, i64) nofree nosync nounwind willreturn
declare i64 @hashmap_keys(ptr) nofree nosync nounwind willreturn
declare i64 @hashmap_values(ptr) nofree nosync nounwind willreturn
declare void @vec_dedup(ptr) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @vec_fill(ptr, i64) nofree nosync nounwind willreturn memory(argmem: readwrite)
declare i64 @vec_copy(ptr) nofree nosync nounwind willreturn
declare i64 @vec_sum(ptr) nofree nosync nounwind willreturn
declare i64 @strmap_new() nofree nosync nounwind willreturn
declare void @strmap_free(i64) nosync nounwind willreturn
declare i64 @strmap_insert(i64, ptr nocapture readonly, i64) nofree nosync nounwind willreturn
declare i64 @strmap_get(i64, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @strmap_contains(i64, ptr nocapture readonly) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @strmap_size(i64) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @strmap_remove(i64, ptr nocapture readonly) nofree nosync nounwind willreturn
declare i64 @strmap_keys(i64) nofree nosync nounwind willreturn
declare i64 @strmap_values(i64) nofree nosync nounwind willreturn
declare i64 @bmb_read_int() nofree nounwind willreturn
declare i64 @vec_min(ptr) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @vec_max(ptr) nofree nosync nounwind willreturn memory(argmem: read)
declare i64 @vec_product(ptr) nofree nosync nounwind willreturn memory(argmem: read)
declare void @bmb_panic_bounds(i64, i64) noreturn nounwind
declare void @bmb_panic_divzero() noreturn nounwind
declare void @llvm.assume(i1 noundef)
@str_data_0 = private unnamed_addr constant [2 x i8] c".\00"
@str_bmb_0 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_0, i64 1, i64 1 }
@str_data_1 = private unnamed_addr constant [24 x i8] c"__nonexistent_dir_xyz__\00"
@str_bmb_1 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_1, i64 23, i64 23 }
@str_data_2 = private unnamed_addr constant [20 x i8] c"__test_dir_golden__\00"
@str_bmb_2 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_2, i64 19, i64 19 }
@str_data_3 = private unnamed_addr constant [7 x i8] c"rmdir \00"
@str_bmb_3 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_3, i64 6, i64 6 }
@str_data_4 = private unnamed_addr constant [29 x i8] c"__test_dir_nested__/sub/deep\00"
@str_bmb_4 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_4, i64 28, i64 28 }
@str_data_5 = private unnamed_addr constant [35 x i8] c"rmdir __test_dir_nested__/sub/deep\00"
@str_bmb_5 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_5, i64 34, i64 34 }
@str_data_6 = private unnamed_addr constant [30 x i8] c"rmdir __test_dir_nested__/sub\00"
@str_bmb_6 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_6, i64 29, i64 29 }
@str_data_7 = private unnamed_addr constant [26 x i8] c"rmdir __test_dir_nested__\00"
@str_bmb_7 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_7, i64 25, i64 25 }
@str_data_8 = private unnamed_addr constant [9 x i8] c"  PASS: \00"
@str_bmb_8 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_8, i64 8, i64 8 }
@str_data_9 = private unnamed_addr constant [9 x i8] c"  FAIL: \00"
@str_bmb_9 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_9, i64 8, i64 8 }
@str_data_10 = private unnamed_addr constant [2 x i8] c"\0A\00"
@str_bmb_10 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_10, i64 1, i64 1 }
@str_data_11 = private unnamed_addr constant [24 x i8] c"dir-ops: running tests\0A\00"
@str_bmb_11 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_11, i64 23, i64 23 }
@str_data_12 = private unnamed_addr constant [19 x i8] c"is_dir on existing\00"
@str_bmb_12 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_12, i64 18, i64 18 }
@str_data_13 = private unnamed_addr constant [22 x i8] c"is_dir on nonexistent\00"
@str_bmb_13 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_13, i64 21, i64 21 }
@str_data_14 = private unnamed_addr constant [19 x i8] c"make_dir and check\00"
@str_bmb_14 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_14, i64 18, i64 18 }
@str_data_15 = private unnamed_addr constant [18 x i8] c"list_dir nonempty\00"
@str_bmb_15 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_15, i64 17, i64 17 }
@str_data_16 = private unnamed_addr constant [21 x i8] c"list_dir nonexistent\00"
@str_bmb_16 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_16, i64 20, i64 20 }
@str_data_17 = private unnamed_addr constant [16 x i8] c"make_dir nested\00"
@str_bmb_17 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_17, i64 15, i64 15 }
@str_data_18 = private unnamed_addr constant [10 x i8] c"dir-ops: \00"
@str_bmb_18 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_18, i64 9, i64 9 }
@str_data_19 = private unnamed_addr constant [2 x i8] c"/\00"
@str_bmb_19 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_19, i64 1, i64 1 }
@str_data_20 = private unnamed_addr constant [15 x i8] c" tests passed\0A\00"
@str_bmb_20 = private unnamed_addr constant { ptr, i64, i64 } { ptr @str_data_20, i64 14, i64 14 }

define private i64 @test_is_dir_on_existing() norecurse alwaysinline mustprogress nounwind willreturn nosync {
entry:
  %_t1 = call i64 @is_dir(ptr @str_bmb_0)
  %_t3_cmp = icmp eq i64 %_t1, 1
  %_t3 = zext i1 %_t3_cmp to i64
  ret i64 %_t3
}

define private i64 @test_is_dir_on_nonexistent() norecurse alwaysinline mustprogress nounwind willreturn nosync {
entry:
  %_t1 = call i64 @is_dir(ptr @str_bmb_1)
  %_t3_cmp = icmp eq i64 %_t1, 0
  %_t3 = zext i1 %_t3_cmp to i64
  ret i64 %_t3
}

define private i64 @test_make_dir_and_check() norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t0 = ptrtoint ptr @str_bmb_2 to i64
  %path_v0 = alloca i64
  store i64 %_t0, ptr %path_v0
  %_t2 = call i64 @make_dir(i64 %_t0)
  %result_v1 = alloca i64
  store i64 %_t2, ptr %result_v1
  %_t3 = load i64, ptr %path_v0
  %_t3_p0 = inttoptr i64 %_t3 to ptr
  %_t4 = call i64 @is_dir(ptr %_t3_p0)
  %_t6_cmp = icmp eq i64 %_t4, 1
  %_t6 = zext i1 %_t6_cmp to i64
  %check_v3 = alloca i64
  store i64 %_t6, ptr %check_v3
  %_t7 = ptrtoint ptr @str_bmb_3 to i64
  %_t8 = load i64, ptr %path_v0
  %_t9 = call i64 @str_concat(i64 %_t7, i64 %_t8)
  %_t9_p0 = inttoptr i64 %_t9 to ptr
  %_t10 = call i64 @bmb_system(ptr %_t9_p0)
  %_t11 = load i64, ptr %result_v1
  %_t13_cmp = icmp eq i64 %_t11, 0
  %_t13 = zext i1 %_t13_cmp to i64
  %_t14 = load i64, ptr %check_v3
  %_t15 = and i64 %_t13, %_t14
  ret i64 %_t15
}

define private i64 @test_list_dir_nonempty() norecurse alwaysinline mustprogress nounwind willreturn nosync {
entry:
  %_t1_ptr = call ptr @list_dir(ptr @str_bmb_0)
  %_t3_lp = getelementptr {ptr, i64, i64}, ptr %_t1_ptr, i32 0, i32 1
  %_t3 = load i64, ptr %_t3_lp, !invariant.load !0
  %_t5_cmp = icmp sgt i64 %_t3, 0
  %_t5 = zext i1 %_t5_cmp to i64
  ret i64 %_t5
}

define private i64 @test_list_dir_nonexistent() norecurse alwaysinline mustprogress nounwind willreturn nosync {
entry:
  %_t1_ptr = call ptr @list_dir(ptr @str_bmb_1)
  %_t3_lp = getelementptr {ptr, i64, i64}, ptr %_t1_ptr, i32 0, i32 1
  %_t3 = load i64, ptr %_t3_lp, !invariant.load !0
  %_t5_cmp = icmp eq i64 %_t3, 0
  %_t5 = zext i1 %_t5_cmp to i64
  ret i64 %_t5
}

define private i64 @test_make_dir_nested() norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t0 = ptrtoint ptr @str_bmb_4 to i64
  %path_v0 = alloca i64
  store i64 %_t0, ptr %path_v0
  %_t2 = call i64 @make_dir(i64 %_t0)
  %result_v1 = alloca i64
  store i64 %_t2, ptr %result_v1
  %_t3 = load i64, ptr %path_v0
  %_t3_p0 = inttoptr i64 %_t3 to ptr
  %_t4 = call i64 @is_dir(ptr %_t3_p0)
  %_t6_cmp = icmp eq i64 %_t4, 1
  %_t6 = zext i1 %_t6_cmp to i64
  %check_v3 = alloca i64
  store i64 %_t6, ptr %check_v3
  %_t8 = call i64 @bmb_system(ptr @str_bmb_5)
  %_t10 = call i64 @bmb_system(ptr @str_bmb_6)
  %_t12 = call i64 @bmb_system(ptr @str_bmb_7)
  %_t13 = load i64, ptr %result_v1
  %_t15_cmp = icmp eq i64 %_t13, 0
  %_t15 = zext i1 %_t15_cmp to i64
  %_t16 = load i64, ptr %check_v3
  %_t17 = and i64 %_t15, %_t16
  ret i64 %_t17
}

define private i64 @str_concat(i64 noundef %a, i64 noundef %b) norecurse alwaysinline mustprogress nounwind willreturn nosync {
entry:
  %_t0 = call i64 @bmb_sb_new()
  %sb_v0 = alloca i64
  store i64 %_t0, ptr %sb_v0
  %_t2_p1 = inttoptr i64 %a to ptr
  %_t3 = call i64 @bmb_sb_push(i64 %_t0, ptr %_t2_p1)
  %_t4 = load i64, ptr %sb_v0
  %_t5_p1 = inttoptr i64 %b to ptr
  %_t6 = call i64 @bmb_sb_push(i64 %_t4, ptr %_t5_p1)
  %_t7 = load i64, ptr %sb_v0
  %_t8_ptr = call ptr @bmb_sb_build(i64 %_t7)
  %_t8 = ptrtoint ptr %_t8_ptr to i64
  ret i64 %_t8
}

define private i64 @run_test(i64 noundef %name, i64 noundef %result) norecurse inlinehint mustprogress nounwind willreturn nosync {
entry:
  %_t0_i1 = trunc i64 %result to i1
  br i1 %_t0_i1, label %then_0, label %else_0
then_0:
  %_t1 = ptrtoint ptr @str_bmb_8 to i64
  br label %merge_0
else_0:
  %_t2 = ptrtoint ptr @str_bmb_9 to i64
  br label %merge_0
merge_0:
  %_t3 = phi i64 [ %_t1, %then_0 ], [ %_t2, %else_0 ]
  %_t3_p0 = inttoptr i64 %_t3 to ptr
  call void @print_str(ptr %_t3_p0)
  %_t5_p0 = inttoptr i64 %name to ptr
  call void @print_str(ptr %_t5_p0)
  call void @print_str(ptr @str_bmb_10)
  %_t12_s = trunc i64 %result to i1
  %_t12 = select i1 %_t12_s, i64 1, i64 0
  ret i64 %_t12
}

define i64 @bmb_user_main() norecurse mustprogress {
entry:
  call void @print_str(ptr @str_bmb_11)
  %passed_2 = alloca i64
  store i64 0, ptr %passed_2
  %total_v3 = alloca i64
  store i64 6, ptr %total_v3
  %_t5 = ptrtoint ptr @str_bmb_12 to i64
  %_t6 = call i64 @test_is_dir_on_existing()
  %_t7 = call i64 @run_test(i64 %_t5, i64 %_t6)
  %_t8 = add nsw i64 0, %_t7
  store i64 %_t8, ptr %passed_2
  %_t11 = ptrtoint ptr @str_bmb_13 to i64
  %_t12 = call i64 @test_is_dir_on_nonexistent()
  %_t13 = call i64 @run_test(i64 %_t11, i64 %_t12)
  %_t14 = add nsw i64 %_t8, %_t13
  store i64 %_t14, ptr %passed_2
  %_t17 = ptrtoint ptr @str_bmb_14 to i64
  %_t18 = call i64 @test_make_dir_and_check()
  %_t19 = call i64 @run_test(i64 %_t17, i64 %_t18)
  %_t20 = add nsw i64 %_t14, %_t19
  store i64 %_t20, ptr %passed_2
  %_t23 = ptrtoint ptr @str_bmb_15 to i64
  %_t24 = call i64 @test_list_dir_nonempty()
  %_t25 = call i64 @run_test(i64 %_t23, i64 %_t24)
  %_t26 = add nsw i64 %_t20, %_t25
  store i64 %_t26, ptr %passed_2
  %_t29 = ptrtoint ptr @str_bmb_16 to i64
  %_t30 = call i64 @test_list_dir_nonexistent()
  %_t31 = call i64 @run_test(i64 %_t29, i64 %_t30)
  %_t32 = add nsw i64 %_t26, %_t31
  store i64 %_t32, ptr %passed_2
  %_t35 = ptrtoint ptr @str_bmb_17 to i64
  %_t36 = call i64 @test_make_dir_nested()
  %_t37 = call i64 @run_test(i64 %_t35, i64 %_t36)
  %_t38 = add nsw i64 %_t32, %_t37
  store i64 %_t38, ptr %passed_2
  call void @print_str(ptr @str_bmb_18)
  %_t42 = load i64, ptr %passed_2
  call void @bmb_print_i64(i64 %_t42)
  call void @print_str(ptr @str_bmb_19)
  %_t46 = load i64, ptr %total_v3
  call void @bmb_print_i64(i64 %_t46)
  call void @print_str(ptr @str_bmb_20)
  %_t50 = load i64, ptr %passed_2
  %_t51 = load i64, ptr %total_v3
  %_t52_cmp = icmp eq i64 %_t50, %_t51
  %_t55 = select i1 %_t52_cmp, i64 0, i64 1
  ret i64 %_t55
}

!0 = !{}