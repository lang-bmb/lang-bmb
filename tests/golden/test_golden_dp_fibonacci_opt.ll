; ModuleID = 'tests/golden/test_golden_dp_fibonacci.ll'
source_filename = "tests/golden/test_golden_dp_fibonacci.ll"

; Function Attrs: mustprogress nofree nounwind willreturn
declare void @println(i64) local_unnamed_addr #0

; Function Attrs: mustprogress nofree norecurse nounwind willreturn
define noundef i64 @bmb_user_main() local_unnamed_addr #1 {
entry:
  tail call void @println(i64 55)
  tail call void @println(i64 6765)
  tail call void @println(i64 8)
  tail call void @println(i64 89)
  tail call void @println(i64 4)
  tail call void @println(i64 9)
  tail call void @println(i64 44)
  tail call void @println(i64 149)
  ret i64 0
}

attributes #0 = { mustprogress nofree nounwind willreturn }
attributes #1 = { mustprogress nofree norecurse nounwind willreturn }
