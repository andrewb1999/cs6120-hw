# Static Single Assigment

### To SSA

There were definitely some complexities when trying to implement the pseudo code to cover all edge cases. As many other people have mentioned here, these issues mainly arise from variables being undefined along some paths and function arguments. For a while I was confusing myself over why a bunch of phi nodes were being generated for variables that were only ever assigned once, but this turns out to just be a side effect of the algorithm and these phi nodes can be automatically optimized away with tdce.

When varibles are undefined along some path, I just add an `__undefined` argument. Overall, SSA, unlike many of the previous implementation exercises, is mostly a challenge of handling edge cases without much concern over how to implement some data structure or framework.

### From SSA

In comparison to converting to ssa, converting out of ssa is relatively simple, at least in the naive case. I followed the naive algorithm of adding copy instructions and deleting phi nodes.

### Testing

I tested using the to_ssa and ssa_roundtrip test cases. I ensured the tests worked with a mix of spot checking and roundtrip testing. I ran out of time to do a more detailed analysis and brench, but my code seems to work for all the cases I have spot checked.

### Usage

```
-f = from_ssa
-t = to_ssa
-r = roundtrip
```
