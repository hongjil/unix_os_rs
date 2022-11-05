```gdb
si        		single step at machine level
x/10i $pc       print next 10 steps starting from the address $pc
p/x $t0         print value in register t0

b *0x80200000   add a break on specific address(i.e. 0x80200000)
b function
b file:line

c               continue to run until a break

watch $fp==0x1  add a break on a specific condition


############## REMOVE below before checked in ######################
# Break before pushing an application's section to the memory set for mapping
b src/mm/memory_set.rs:225  

# Break before mapping the page into pagetable
b src/mm/page_table.rs:94


file ../user/target/riscv64gc-unknown-none-elf/release/00power_3


# b __restore
si 39


b trap_return
0xfffffffffffff000
0xffffffffffffe000

0x0000000000010000

d delete all breakpoints


```


