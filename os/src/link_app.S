
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad 26
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_6_start
    .quad app_7_start
    .quad app_8_start
    .quad app_9_start
    .quad app_10_start
    .quad app_11_start
    .quad app_12_start
    .quad app_13_start
    .quad app_14_start
    .quad app_15_start
    .quad app_16_start
    .quad app_17_start
    .quad app_18_start
    .quad app_19_start
    .quad app_20_start
    .quad app_21_start
    .quad app_22_start
    .quad app_23_start
    .quad app_24_start
    .quad app_25_start
    .quad app_25_end

    .global _app_names
_app_names:
    .string "00hello_world"
    .string "01store_fault"
    .string "02power"
    .string "03priv_inst"
    .string "04load_fault"
    .string "04priv_csr"
    .string "05sleep"
    .string "exit"
    .string "fantastic_text"
    .string "forkexec"
    .string "forktest"
    .string "forktest2"
    .string "forktest_simple"
    .string "forktree"
    .string "hello_world"
    .string "initproc"
    .string "matrix"
    .string "sbrk_test"
    .string "sleep"
    .string "sleep_simple"
    .string "stack_overflow"
    .string "test"
    .string "user_shell"
    .string "usertests"
    .string "usertests-simple"
    .string "yield"

    .section .data
    .global app_0_start
    .global app_0_end
    .align 3
app_0_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/00hello_world"
app_0_end:

    .section .data
    .global app_1_start
    .global app_1_end
    .align 3
app_1_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/01store_fault"
app_1_end:

    .section .data
    .global app_2_start
    .global app_2_end
    .align 3
app_2_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/02power"
app_2_end:

    .section .data
    .global app_3_start
    .global app_3_end
    .align 3
app_3_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/03priv_inst"
app_3_end:

    .section .data
    .global app_4_start
    .global app_4_end
    .align 3
app_4_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/04load_fault"
app_4_end:

    .section .data
    .global app_5_start
    .global app_5_end
    .align 3
app_5_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/04priv_csr"
app_5_end:

    .section .data
    .global app_6_start
    .global app_6_end
    .align 3
app_6_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/05sleep"
app_6_end:

    .section .data
    .global app_7_start
    .global app_7_end
    .align 3
app_7_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/exit"
app_7_end:

    .section .data
    .global app_8_start
    .global app_8_end
    .align 3
app_8_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/fantastic_text"
app_8_end:

    .section .data
    .global app_9_start
    .global app_9_end
    .align 3
app_9_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/forkexec"
app_9_end:

    .section .data
    .global app_10_start
    .global app_10_end
    .align 3
app_10_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/forktest"
app_10_end:

    .section .data
    .global app_11_start
    .global app_11_end
    .align 3
app_11_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/forktest2"
app_11_end:

    .section .data
    .global app_12_start
    .global app_12_end
    .align 3
app_12_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/forktest_simple"
app_12_end:

    .section .data
    .global app_13_start
    .global app_13_end
    .align 3
app_13_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/forktree"
app_13_end:

    .section .data
    .global app_14_start
    .global app_14_end
    .align 3
app_14_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/hello_world"
app_14_end:

    .section .data
    .global app_15_start
    .global app_15_end
    .align 3
app_15_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/initproc"
app_15_end:

    .section .data
    .global app_16_start
    .global app_16_end
    .align 3
app_16_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/matrix"
app_16_end:

    .section .data
    .global app_17_start
    .global app_17_end
    .align 3
app_17_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/sbrk_test"
app_17_end:

    .section .data
    .global app_18_start
    .global app_18_end
    .align 3
app_18_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/sleep"
app_18_end:

    .section .data
    .global app_19_start
    .global app_19_end
    .align 3
app_19_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/sleep_simple"
app_19_end:

    .section .data
    .global app_20_start
    .global app_20_end
    .align 3
app_20_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/stack_overflow"
app_20_end:

    .section .data
    .global app_21_start
    .global app_21_end
    .align 3
app_21_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/test"
app_21_end:

    .section .data
    .global app_22_start
    .global app_22_end
    .align 3
app_22_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/user_shell"
app_22_end:

    .section .data
    .global app_23_start
    .global app_23_end
    .align 3
app_23_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/usertests"
app_23_end:

    .section .data
    .global app_24_start
    .global app_24_end
    .align 3
app_24_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/usertests-simple"
app_24_end:

    .section .data
    .global app_25_start
    .global app_25_end
    .align 3
app_25_start:
    .incbin "../user/target/riscv64gc-unknown-none-elf/release/yield"
app_25_end:
