```gdb
si        		single step at machine level
x/10i $pc       print next 10 steps starting from the address $pc
p/x $t0         print value in register t0

b *0x80200000   add a break on specific address(i.e. 0x80200000)
b function      ... on a function
b file:line     ... on a line of file
d               delete all breakpoints


c               continue to run until a break

watch $fp==0x1  add a break on a specific condition
```


