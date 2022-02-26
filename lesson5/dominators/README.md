## Lesson 5 Tasks: Dominance Utilities

### Find Dominators
I provide a function find_dominators to find the dominators of a given CFG. This function uses the algorithm given in class to find dominators. Vertices are iterated in reverse post-order to achieve linear runtime on reducible CFGs. Special care is required for the entry node, which must be initialized correctly and not modified during the iteration. Output is given as a HashMap from String to a HashSet of Strings that provides the dominators for that block. find_dominators_num is also provided which returns the same output, but with block numbers from the CFG rather than block names. This method makes it easier to access information about a block in the CFG.

### Form Dominance Tree
The dominance tree is represented as a pair of rust structs, using the arena allocation pattern. Each tree node has a block label, a parent index, and a vector of child indices. The tree is constructed by first finding the immediate dominators, which is determined using the definition directly, then using the immediate dominators to construct edges in the tree.

### Get Dominance Frontier
The dominance frontier is computed directly using the definition that A's dominance frontier contains B iff A does not strictly dominate B, but A does dominate some predecessor of B. The dominance frontier is also represented as a HashMap from a String to a HashSet of Strings.

### Testing
A testing framework for dominators is built in `test.rs`. It compares the computed domiators to the naive algorithm for determining if one block dominates the other, by computing all paths from the entry node to B and seeing if A is in all those paths. This is slow compared to the real dominators algorithm, but still runs within 0.2s user time for small test cases (thanks Rust!).

### Arena Allocation
One of the more challenging parts of this assignment for we was how to represent the dominance tree correctly in Rust. I struggled for a bit trying to use a more C-like style to point tree nodes to each other, but ended up learning about the wonders of arena allocated data structures. In this pattern, all nodes are owned by a vector in a parent struct, and then the "pointers" are actually just indexes into the vector that owns all the nodes. Using this pattern you are able to completely avoid the challenges of ownership and borrowing that arise in tree-like data structures.
