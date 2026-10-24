// A small, deterministic D3D11 workload for the PresentMon integration probe.
// No hooks, injection or changes to an existing application.
#include <windows.h>
#include <d3d11.h>
#include <dxgi.h>
#include <wrl/client.h>
#include <chrono>
#include <cmath>
#include <cstdlib>
#include <iostream>
#include <sstream>
#include <thread>

using Microsoft::WRL::ComPtr;
using Clock = std::chrono::steady_clock;

LRESULT CALLBACK WindowProc(HWND window, UINT message, WPARAM wp, LPARAM lp) {
    if (message == WM_CLOSE) { DestroyWindow(window); return 0; }
    if (message == WM_DESTROY) { PostQuitMessage(0); return 0; }
    return DefWindowProcW(window, message, wp, lp);
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR arguments, int) {
    const double duration = arguments[0] ? _wtof(arguments) : 30.0;
    if (duration < 1 || duration > 3600) return 2;
    WNDCLASSW wc{};
    wc.hInstance = instance;
    wc.lpfnWndProc = WindowProc;
    wc.lpszClassName = L"BezelFpsEvaluation";
    if (!RegisterClassW(&wc)) return 3;
    HWND window = CreateWindowExW(WS_EX_TOPMOST | WS_EX_NOACTIVATE,
        wc.lpszClassName, L"Bezel FPS evaluation - 60 / 30 / 60",
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU, 40, 80, 520, 280,
        nullptr, nullptr, instance, nullptr);
    if (!window) return 4;
    DXGI_SWAP_CHAIN_DESC desc{};
    desc.BufferCount = 2;
    desc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    desc.OutputWindow = window;
    desc.SampleDesc.Count = 1;
    desc.Windowed = TRUE;
    desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
    ComPtr<ID3D11Device> device;
    ComPtr<ID3D11DeviceContext> context;
    ComPtr<IDXGISwapChain> chain;
    HRESULT hr = D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_HARDWARE,
        nullptr, 0, nullptr, 0, D3D11_SDK_VERSION, &desc, &chain, &device,
        nullptr, &context);
    if (FAILED(hr)) return 5;
    ComPtr<ID3D11Texture2D> texture;
    ComPtr<ID3D11RenderTargetView> target;
    if (FAILED(chain->GetBuffer(0, IID_PPV_ARGS(&texture))) ||
        FAILED(device->CreateRenderTargetView(texture.Get(), nullptr, &target))) return 6;
    ShowWindow(window, SW_SHOWNOACTIVATE);
    HANDLE timer = CreateWaitableTimerExW(nullptr, nullptr, 2, TIMER_ALL_ACCESS);
    if (!timer) timer = CreateWaitableTimerW(nullptr, FALSE, nullptr);
    const auto start = Clock::now();
    auto next = start;
    unsigned long long frames = 0;
    int result = 0;
    while (true) {
        MSG message{};
        while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) {
            if (message.message == WM_QUIT) goto finished;
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        const double elapsed = std::chrono::duration<double>(Clock::now() - start).count();
        if (elapsed >= duration) break;
        const int phase = static_cast<int>(elapsed / 4.0) % 3;
        const double fps = phase == 1 ? 30.0 : 60.0;
        const float color[] = {0.08f, phase == 1 ? 0.15f : 0.55f,
            0.25f + 0.15f * static_cast<float>(std::sin(elapsed)), 1.0f};
        context->ClearRenderTargetView(target.Get(), color);
        hr = chain->Present(0, 0);
        if (FAILED(hr)) { result = 7; break; }
        if (hr != DXGI_STATUS_OCCLUDED) frames++;
        next += std::chrono::duration_cast<Clock::duration>(std::chrono::duration<double>(1.0 / fps));
        const auto delay = next - Clock::now();
        if (delay > Clock::duration::zero() && timer) {
            LARGE_INTEGER due{};
            due.QuadPart = -std::chrono::duration_cast<std::chrono::nanoseconds>(delay).count() / 100;
            if (due.QuadPart == 0) due.QuadPart = -1;
            SetWaitableTimer(timer, &due, 0, nullptr, nullptr, FALSE);
            WaitForSingleObject(timer, 1000);
        }
    }
finished:
    if (timer) CloseHandle(timer);
    if (IsWindow(window)) DestroyWindow(window);
    const double seconds = std::chrono::duration<double>(Clock::now() - start).count();
    std::ostringstream summary;
    summary << "{\"seconds\":" << seconds << ",\"successful_presents\":" << frames
        << ",\"present_rate\":" << frames / seconds << "}\n";
    const auto text = summary.str();
    DWORD written = 0;
    WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), text.data(),
        static_cast<DWORD>(text.size()), &written, nullptr);
    return result;
}
