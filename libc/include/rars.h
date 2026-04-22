#ifndef H_RARC_LIBC_RARS

#define H_RARC_LIBC_RARS

#define RARS_PrintInt            1
#define RARS_PrintFloat          2
#define RARS_PrintDouble         3
#define RARS_PrintString         4
#define RARS_ReadInt             5
#define RARS_ReadFloat           6
#define RARS_ReadDouble          7
#define RARS_ReadString          8
#define RARS_Sbrk                9
#define RARS_Exit                10
#define RARS_PrintChar           11
#define RARS_ReadChar            12
#define RARS_GetCWD              17
#define RARS_Time                30
#define RARS_MidiOut             31
#define RARS_Sleep               32
#define RARS_MidiOutSync         33
#define RARS_PrintIntHex         34
#define RARS_PrintIntBinary      35
#define RARS_PrintIntUnsigned    36
#define RARS_RandSeed            40
#define RARS_RandInt             41
#define RARS_RandIntRange        42
#define RARS_RandFloat           43
#define RARS_RandDouble          44
#define RARS_ConfirmDialog       50
#define RARS_InputDialogInt      51
#define RARS_InputDialogFloat    52
#define RARS_InputDialogDouble   53
#define RARS_InputDialogString   54
#define RARS_MessageDialog       55
#define RARS_MessageDialogInt    56
#define RARS_Close               57
#define RARS_MessageDialogDouble 58
#define RARS_MessageDialogString 59
#define RARS_MessageDialogFloat  60
#define RARS_LSeek               62
#define RARS_Read                63
#define RARS_Write               64
#define RARS_Exit2               93
#define RARS_Open                1024


unsigned long syscall(long, ...);
#ifndef _RARC_STANDALONE
#include "../src/rars.c"
#endif

#endif