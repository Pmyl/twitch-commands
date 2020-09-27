#include <Windows.h>

extern "C"

int is_mouse_pressed() {
   return (GetKeyState(VK_LBUTTON) & 0x80) != 0 ? 1 : 0;
}
