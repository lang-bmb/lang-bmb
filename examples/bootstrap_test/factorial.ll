; ModuleID = bmb_bootstrap
; Generated following bootstrap/llvm_ir.bmb patterns
target triple = "x86_64-pc-windows-msvc"

; Runtime declarations
declare void @println(i64)
declare i64 @abs(i64)
declare i64 @min(i64, i64)
declare i64 @max(i64, i64)

; fn factorial_iter(n: i64, acc: i64) -> i64 =
;     if n <= 1 then acc
;     else factorial_iter(n - 1, acc * n);
define i64 @factorial_iter(i64 %n, i64 %acc) {
entry:
  ; n <= 1
  %cmp = icmp sle i64 %n, 1
  br i1 %cmp, label %then_0, label %else_0

then_0:
  ; return acc
  br label %merge_0

else_0:
  ; n - 1
  %n_minus_1 = sub i64 %n, 1
  ; acc * n
  %new_acc = mul i64 %acc, %n
  ; factorial_iter(n - 1, acc * n)
  %rec_result = call i64 @factorial_iter(i64 %n_minus_1, i64 %new_acc)
  br label %merge_0

merge_0:
  %result = phi i64 [ %acc, %then_0 ], [ %rec_result, %else_0 ]
  ret i64 %result
}

; fn factorial(n: i64) -> i64 = factorial_iter(n, 1);
define i64 @factorial(i64 %n) {
entry:
  %result = call i64 @factorial_iter(i64 %n, i64 1)
  ret i64 %result
}

; fn main() -> i64 =
;     let result = factorial(5);
;     let u0 = println(result);
;     0;
define i64 @main() {
entry:
  %result = call i64 @factorial(i64 5)
  call void @println(i64 %result)
  ret i64 0
}
