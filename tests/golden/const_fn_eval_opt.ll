; ModuleID = 'tests/golden/const_fn_eval.ll'
source_filename = "tests/golden/const_fn_eval.ll"

; Function Attrs: mustprogress nofree nounwind willreturn
declare void @println(i64) local_unnamed_addr #0

; Function Attrs: mustprogress nofree norecurse nounwind willreturn
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

attributes #0 = { mustprogress nofree nounwind willreturn }
attributes #1 = { mustprogress nofree norecurse nounwind willreturn }
