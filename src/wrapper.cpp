#include <Windows.h>

extern "C"
{
	DWORD _c_init_cs(LPCRITICAL_SECTION lpCriticalSection)
	{
		try
		{
			InitializeCriticalSection(lpCriticalSection);
			return 1;
		}
		catch (...)
		{
			return 0;
		}
	}
	DWORD _c_init_cs_with_spin_count(LPCRITICAL_SECTION lpCriticalSection, DWORD spin_count)
	{
		return InitializeCriticalSectionAndSpinCount(lpCriticalSection, spin_count);
	}
	DWORD _c_enter_cs(LPCRITICAL_SECTION lpCriticalSection)
	{
		try
		{
			EnterCriticalSection(lpCriticalSection);
			return 1;
		}
		catch (...)
		{
			return 0;
		}
	}
    DWORD _c_try_enter_cs(LPCRITICAL_SECTION lpCriticalSection)
    {
        return TryEnterCriticalSection(lpCriticalSection);
    }
	void _c_leave_cs(LPCRITICAL_SECTION lpCriticalSection)
	{
		return LeaveCriticalSection(lpCriticalSection);
	}
	void _c_delete_cs(LPCRITICAL_SECTION lpCriticalSection)
	{
		return DeleteCriticalSection(lpCriticalSection);
	}
	DWORD _c_set_cs_spin_count(LPCRITICAL_SECTION lpCriticalSection, DWORD spin_count)
	{
		return SetCriticalSectionSpinCount(lpCriticalSection, spin_count);
	}
}