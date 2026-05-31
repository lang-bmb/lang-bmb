; ModuleID = 'tests/golden/const_fn_eval.ll'
source_filename = "tests/golden/const_fn_eval.ll"

; Function Attrs: mustprogress nofree nounwind willreturn
declare void @println(i64) local_unnamed_addr #0

; Function Attrs: alwaysinline mustprogress nofree norecurse nosync nounwind willreturn uwtable
define noundef i64 @bmb_user_main() local_unnamed_addr #1 {
entry:
  tail call void @println(i64 42)
  tail call void @println(i64 42)
  tail call void @println(i64 84)
  tail call void @println(i64 0)
  tail call void @println(i64 314)
  tail call void @println(i64 314)
  tail call void @println(i64 628)
  ret i64 0
}

declare void @bmb_init_runtime(i32, ptr) local_unnamed_addr

declare void @bmb_arena_destroy() local_unnamed_addr

; Function Attrs: nounwind uwtable
define noundef i32 @main(i32 %argc, ptr %argv) local_unnamed_addr #2 {
  tail call void @bmb_init_runtime(i32 %argc, ptr %argv) #3
  tail call void @println(i64 42)
  tail call void @println(i64 42)
  tail call void @println(i64 84)
  tail call void @println(i64 0)
  tail call void @println(i64 314)
  tail call void @println(i64 314)
  tail call void @println(i64 628)
  tail call void @bmb_arena_destroy() #3
  ret i32 0
}

attributes #0 = { mustprogress nofree nounwind willreturn }
attributes #1 = { alwaysinline mustprogress nofree norecurse nosync nounwind willreturn uwtable "no-trapping-math"="true" }
attributes #2 = { nounwind uwtable "no-trapping-math"="true" }
attributes #3 = { nounwind }
