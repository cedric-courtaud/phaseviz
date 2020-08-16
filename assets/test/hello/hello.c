#include "valgrind.h"

#include <stdio.h>

#include <unistd.h>
#include <sys/types.h> 
#include <sys/wait.h>

int main() {

    pid_t pid = fork();

    VALGRIND_MONITOR_COMMAND("checkpoint Before_hello");           
    
    if (pid != 0) {
        printf("Parent says \"Hello world!\"\n");
        wait(NULL);
    } else {
        printf("Child says \"Hello world!\"\n");
    }

    VALGRIND_MONITOR_COMMAND("checkpoint After_hello");           

    return 0;
}