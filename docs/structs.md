Structs is Masterminds equivalent of C structs it is a method of grouping multiple related variables together. 
Since Mastermind only supports one primitive data type this means that Structs are collections of one or more cell data types (single or array cells).

Structs can be used to mimic the styles of object orientated code by adding a struct and a series of related functions that 
interface with specific structs to almost form a set of class methods. This forms the basis for Masterminds standard library.

Below is example code for how to use Structs

```
struct box {
  cell height;
  cell width;
}

def area(struct box boxArg, cell output) {
    output = 0;
    // height * width
    copy boxArg.width {
        output += boxArg.height
    }
}

sturct box MyBox;
MyBox.height = 12;
MyBox.width = 13;
cell area;
area(MyBox, area);
//area should now equal 156
```

Below is an example for a struct using arrays

```
struct grid {
  cell x1[3];
  cell x2[3];
  cell x3[3];
}
```