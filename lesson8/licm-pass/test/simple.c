#define COUNT 100000
#include <stdio.h>

int main () {
  int arr[COUNT];
  int y = 9;
  int z = 5;
  for (int i = 0; i < COUNT; i++) {
    for (int j = 0; j < COUNT; j++) {
      int x = y * z;
      int a = x + 2;
      int b = a << 2;
      arr[j] = b;
    }
  }
}
