// Originally From: https://elixir.bootlin.com/glibc/glibc-2.38/source/posix/test-vfork.c
// Made it musl compatible

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
// #include <error.h>
#include <errno.h>
#include <sys/wait.h>

// void __attribute_noinline__ noop (void);

#define NR	2	/* Exit code of the child.  */

/*
  The successful output is 
  Before vfork
  Child print something (child).
  After vfork (parent)
*/
int
main (void)
{
  pid_t pid;
  int status;

  printf ("Before vfork\n");
  fflush (stdout);
  pid = vfork ();
  if (pid == 0)
    {
      /* This will clobber the return pc from vfork in the parent on
	 machines where it is stored on the stack, if vfork wasn't
	 implemented correctly, */
      // noop ();
        sleep(1);
        printf("Child print something (child).\n");
      _exit (NR);
    }
  else if (pid < 0) {
    // error (1, errno, "vfork");
    printf("vfork error: %d\n", errno);
    exit(errno);
  }
  printf ("After vfork (parent)\n");
  if (waitpid (0, &status, 0) != pid
      || !WIFEXITED (status) || WEXITSTATUS (status) != NR)
    exit (1);

  return 0;
}

void
noop (void)
{
}

