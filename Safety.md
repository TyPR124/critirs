# Safety Considerations by Function

## InitializeCriticalSection

[Exceptions](#Exceptions)

[Memory Management](#Memory%20Management)

## InitializeCriticalSectionAndSpinCount

[Memory Management](#Memory%20Management)

## EnterCriticalSection

[Exceptions](#Exceptions)

[Memory Management](#Memory%20Management)

[Reentrancy](#Reentrancy)

[Thread Termination](#Thread%20Termination)

[Deletion](#Deletion)

## TryEnterCriticalSection

[Memory Management](#Memory%20Management)

[Reentrancy](#Reentrancy)

[Thread Termination](#Thread%20Termination)

[Deletion](#Deletion)

## LeaveCriticalSection

[Memory Management](#Memory%20Management)

[Reentrancy](#Reentrancy)

[Thread Termination](#Thread%20Termination)

[Leaving Unentered](#Leaving%20Unentered)

## DeleteCriticalSection

[Memory Management](#Memory%20Management)

[Deletion](#Deletion)

## SetCriticalSectionSpinCount

[Memory Management](#Memory%20Management)

# Safety Considerations

## Exceptions

### Problem

The following functions may throw exceptions:

InitializeCriticalSection - Windows XP and Server 2003 may throw STATUS_NO_MEMORY.

EnterCriticalSection - May throw EXCEPTION_POSSIBLE_DEADLOCK. Microsoft specifies this error should not be handled, and instead the application should be debugged.

### Solution

All functions which may throw exceptions are wrapped in wrapper.cpp using try/catch blocks with appropriate return values on failure.

## Memory Management

### Problem

Once initalized, CRITICAL_SECTION objects may not be moved in memory until deleted.

### Solution

CriticalSection uses Arc to heap-allocate the object which is never moved.

All methods on CriticalStatic require &'static self, which ensures the contained CRITICAL_SECTION is not moved.

## Initialization

### Problem

Reinitializing a CRITICAL_SECTION that has not been deleted is undefined behavior.

### Solution

CriticalSection - The object is initialized once at creation and never again.

CriticalStatic - Initializing the object manually is considered unsafe and is not necessary unless delete was called. Delete is also unsafe. In the case of CriticalStaticRef\<Init\>, unsafe is required to delete which creates a non-Copy/Clone CriticalStaticRef\<Uninit\> which can be safely re-initialized exactly once.

## Reentrancy

### Problem

CRITICAL_SECTIONS can be re-entered on the same thread without blocking. A thread will need to call Leave the same number of times it called Enter or TryEnter successfully. By itself, this is not a safety issue, however when paired with the [Thread Termination](#Thread%20Termination) problem, this must be considered for safety.

### Solution

All CriticalSection and related types return an EnteredSection on successful entry. EnteredSection automatically calls Leave on drop, ensuring every successful Enter or TryEnter has a corresponding Leave.

## Thread Termination

### Problem

If a thread terminates while it has entered a critical section, the state of the critical section is undefined.

### Solution

A Rust thread may terminate early via panic. Panic may either unwind the stack while other threads continue to run, or abort the program and all threads. Therefore, we are only concerned with the unwinding panic.

The EnteredSection returned from all successful Enter or TryEnter calls automatically calls Leave. It will get dropped during unwind, ensuring no thread terminates without leaving the critical section.

TODO: Track poisoning of CriticalSection, CriticalStatic.

## Deletion

### Problem

If a critical section is deleted while a thread is entered, the state of threads waiting to enter is undefined.

### Solution

CriticalSection: only way to delete is to drop the underlying Arc, which is not possible while a thread is entered.

CriticalStatic: deleting the critical section is always marked unsafe.

## Leaving Unentered

### Problem

Leaving a critical section that is not entered by the current thread may cause other threads to deadlock waiting for the critical section. This is not a safety consideration, but is documented here anyway.

### Solution

It is not possible to leave a critical section without obtaining an EnteredSection object via Enter or TryEnter first.