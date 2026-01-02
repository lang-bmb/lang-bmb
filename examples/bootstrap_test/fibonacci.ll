; ModuleID = bmb_bootstrap
; Generated following bootstrap/llvm_ir.bmb patterns
target triple = "x86_64-pc-windows-msvc"

; Runtime declarations
declare void @println(i64)
declare i64 @abs(i64)
declare i64 @min(i64, i64)
declare i64 @max(i64, i64)

; fn fib(n: i64) -> i64 =
;     if n <= 1 then n
;     else fib(n - 1) + fib(n - 2);
define i64 @fib(i64 %n) {
entry:
  ; n <= 1
  %cmp = icmp sle i64 %n, 1
  br i1 %cmp, label %then_0, label %else_0

then_0:
  ; return n
  br label %merge_0

else_0:
  ; fib(n - 1)
  %n_minus_1 = sub i64 %n, 1
  %fib_n1 = call i64 @fib(i64 %n_minus_1)
  ; fib(n - 2)
  %n_minus_2 = sub i64 %n, 2
  %fib_n2 = call i64 @fib(i64 %n_minus_2)
  ; fib(n-1) + fib(n-2)
  %sum = add i64 %fib_n1, %fib_n2
  br label %merge_0

merge_0:
  %result = phi i64 [ %n, %then_0 ], [ %sum, %else_0 ]
  ret i64 %result
}

; fn main() -> i64 =
;     let result = fib(10);
;     let u0 = println(result);
;     0;
define i64 @main() {
entry:
  %result = call i64 @fib(i64 10)
  call void @println(i64 %result)
  ret i64 0
}
