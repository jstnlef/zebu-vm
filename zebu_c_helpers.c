#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <assert.h>

// a pointer to the symbol table I generate
extern void* mu_sym_table;

//extern int32_t mu_retval;
//
//int32_t muentry_get_retval(){
//    return mu_retval;
//}

// a function to resolve symbol names to their address, using mu_sym_table\
// which is valued by the c_resolve_sym function, according to \
// mu_sym_table_root
// actually this function will search for the symbol in one idX_sym_table
void * resolve_symbol_in_current_context(const unsigned char * input_symbol_name){
    unsigned long i = 0, j=0;
    unsigned long num_of_syms = 0;
    unsigned long cur_len = 0;
    unsigned long input_len = 0;
    unsigned char cur_char = 0;
    unsigned long cur_add = 0;
    unsigned char is_different = 0;

    while(input_symbol_name[input_len] != 0){
        input_len++;
    }

    if( (mu_sym_table == NULL) || (input_len == 0)){
        printf("**********\n****ERROR!!!****\nC_RESOLVE_SYMBOL: extern mu sym table is null!\n*********\n");
        assert(0);
    }

    // a copy of mu_sym_table, to keep mu_sym_table from changing
//    unsigned char * sym_table_current = (unsigned char *) get_sym_table_pointer();
    unsigned char * sym_table_current = (unsigned char *) &mu_sym_table;
//    printf("C_RESOLVE_SYMBOL --> Step 1\n");

//    printf("C_RESOLVE_SYMBOL --> Current mu_sym_table = %p\n", sym_table_current);
//    printf("C_RESOLVE_SYMBOL --> Current vm = %p\n", get_vm_pointer());

    // read the total number of symbols from the first line of sym table
    num_of_syms = *((unsigned long*)sym_table_current);
//    printf("RESOLVE_SYM --> number of symbols = %d\n", num_of_syms);
    // go 8 bytes forward, to skip the current .quad
    sym_table_current += 8;

    for(i=0; i<num_of_syms; i++){
        is_different = 0;
        // length of the symbol we are going to check
        cur_len = *((unsigned long*)sym_table_current);
        if(cur_len != input_len){
            sym_table_current += 8;     //cur_len
            sym_table_current += 8;     //cur_add
            sym_table_current += cur_len;     //length of cur_name
//            printf("*** Sym name doesn't match! ***\n");
            continue;
        }
        sym_table_current += 8;
//        printf("Sym_Tbl current Len = %d\n", cur_len);
        for(j=0; j<cur_len; j++){
            cur_char = *((unsigned char*)sym_table_current);
            sym_table_current += 1;
//            printf("Sym_Tbl read char = %d , input symbol current char = %d\n", cur_char, input_symbol_name[j]);
            if(cur_char == input_symbol_name[j]){
                continue;
            }
            else{
                is_different = 1;
                sym_table_current += ((cur_len-j)-1);
                break;
            }
        }
        if(is_different == 1){
            // skip the 64b address in current location and continue to the next sym in table
//            printf("*** Sym name doesn't match! ***\n");
            sym_table_current += 8;
            continue;
        }
        else{
//            printf("*** Sym name = %s! Sym Address = %llu***\n", input_symbol_name, *((long*) sym_table_current));
//            printf("*** Last char = %c\n", sym_table_current[-1]);
//            for(j=0; j<8; j++){
//                printf("*** Last char = %c\n", sym_table_current[j]);
//            }

            unsigned long result = 0;
            unsigned long shifter=1;
            unsigned int ii=0, jj=0;

            for (ii=0; ii<8; ii++){
                shifter = 1;
                for(jj=0; jj<ii; jj++)
                    shifter = shifter * 256;
                result += ((sym_table_current[ii]%256)* shifter);
//                printf("Char = %x\n", (sym_table_current[ii]%256));
//                printf("Result = %llu\n", result);
            }

//            printf("*** going to return {-%llu-}\n", result);
            return ((void*) result);
        }
    }
    return NULL;
}

void * c_resolve_symbol(const unsigned char * input_symbol_name){
    return resolve_symbol_in_current_context(input_symbol_name);
}
