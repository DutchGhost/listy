# listy

## Overview
This library provides the implementation
of a singly linked list, and a doubly linked list,
written in Rust.
Both structures are interesting to implement,
because they require carefull ownership management.


## Singly linked lists
A singly linked list is a structure containg nodes, where each node has a value,
and a pointer to another node.
Because each node has a pointer a next node, inserting into, and removing from the list are quick operations *on paper*.

#### Inserting
Inserting into a list requires re-pointing the previous node to the inserted node,
and pointing the newly-inserted node to the next node.
This is a constant operation, so inserting is _O(1)_.
```
Before insert:
    HEAD -> 1 -> 3 -> 4 -> 5 -> None

During insert:
    (point newly inserted node to next):
    HEAD -> 1 -> 3 -> 4 -> 5 -> None
               /
              2

    (re-point previous node)
    HEAD -> 1    3 -> 4 -> 5 -> None
             \  /
              2
After insert:
    HEAD -> 1 -> 2 -> 3 -> 4 -> 5 -> None
```
#### Removing
Removing from a list requires re-pointing the previous node to the next node.
Thus removing is _(1)_
```
Before remove:
    HEAD -> 1 -> 2 -> 3 -> 4 -> 5 -> None

During remove:
    (re-point previous node to next)
    HEAD -> 1 -> 2 -> 4 -> 5 -> None
                     /
                    3
After remove:
    HEAD -> 1 -> 2 -> 4 -> 5 -> None
```

#### Searching
A linked list can typically only be accessed via its head node, and from there be traversed down untill you reach node you seek. Searching is _O(n)_.

#### Summery
The time complexity's are:
 - Inserting: _O(1)_
 - Removing: _O(1)_
 - Searching: _O(n)_

However, in practice a linked list is slow.
Inserting and removing are only constant if you already know where you want to insert/remove.
Most of the time one has to traverse a list to find an insertion or removal node,
and traversing a linked list is slow.

#### The horror of traversing
Nodes are stored incontiguously, greatly increasing the time required to access individual elements within the list, especially with a CPU cache.
The cache consists of cache lines, that each have a size of 64 bytes (on x86).<br>
Loading from memory fills an entire cacheline.
For a linked list, that means a node's element and pointer are loaded into a cacheline.
When accessing the next element trough the pointer, a new load must be performed, wasting
the previous load. This is called a *cache miss*.<br>

Another reason why traversing a linked list is slow, is because of its loop-carried dependency.<br>
We dont know where the next element of the list is, so we have to follow the next pointer.
This is a load to memory, that must complete before we can continue.<br>All in all, to continue to the next iteration, we depend on the current one. This is called *pointer chasing behaviour*, which is inherently slow on modern hardware.<br>
