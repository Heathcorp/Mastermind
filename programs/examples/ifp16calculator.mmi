//THIS CALCULATOR REQUIRES AN INPUT IN A SPECIFIC FORMAT
//Each float seperated by a newline with commands between E.G
// 11.11
// +
// 11.11
// +
// 12.10
// =

// This will execute operations in the order given not BODMAS/PEMDAS

#include <ifp16>
struct ifp16 dig1;
struct ifp16 dig2;
cell symbol;
cell blank;
_0(dig1); read(dig1);
_0(dig2); 
input symbol;
input blank;
cell next_input = 1;
while next_input {
    read(dig2);
    //Calculator stuff in here
     print(dig1);
     // output " ("; debug(dig1); output ")";
     output " ";
     output symbol;
     output " ";
     print(dig2);
     // output " ("; debug(dig2); output ")";
     output " = ";
     if not symbol - '+'{
        add(dig1,dig2);
     }
     if not symbol - '-'{
        sub(dig1,dig2);
     }
     if not symbol - '/'{
        divide(dig1,dig2);
     }
     if not symbol - '*'{
        mult(dig1,dig2);
     }
     print(dig1);
     // output " ("; debug(dig1); output ")";
     input next_input;
     input blank;
     symbol = next_input;
     next_input -= 61;
     output "\n";
}