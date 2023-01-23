#include "hello.h"


static int i = 0;


void delay0(void (*rust_delay)()) {
    rust_delay();
}


int mydelay()
{
    //delay0();
    return i;
}
