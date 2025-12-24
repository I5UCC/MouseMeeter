using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

namespace Mousemeeter;

public class VoicemeeterController
{
    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int VBVMR_Login();

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int VBVMR_Logout();

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int VBVMR_GetParameterFloat([MarshalAs(UnmanagedType.LPStr)] string szParamName, ref float pValue);

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int VBVMR_SetParameterFloat([MarshalAs(UnmanagedType.LPStr)] string szParamName, float Value);

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int VBVMR_SetParameterStringA([MarshalAs(UnmanagedType.LPStr)] string szParamName, [MarshalAs(UnmanagedType.LPStr)] string szString);

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int VBVMR_IsParametersDirty();

    private readonly MousemeeterConfig _config;
    private bool _isConnected = false;

    // Cached volume values
    private readonly Dictionary<int, float> _cachedGainValues = new Dictionary<int, float>(3);
    private readonly Dictionary<int, bool> _cachedMuteValues = new Dictionary<int, bool>(3);

    // Background processing
    private readonly Queue<VolumeAction> _actionQueue = new Queue<VolumeAction>();
    private readonly Lock _queueLock = new Lock();
    private readonly CancellationTokenSource _cancellationTokenSource = new CancellationTokenSource();
    private readonly Task _processingTask;

    public VoicemeeterController(MousemeeterConfig config)
    {
        this._config = config;
        Connect();

        // Start background processing task
        _processingTask = Task.Run(ProcessActionsBackground, _cancellationTokenSource.Token);
    }

    private bool Connect()
    {
        try
        {
            var result = VBVMR_Login();
            _isConnected = result is 0 or 1;

            if (!_isConnected)
            {
                Console.WriteLine("Voicemeeter is not running or failed to connect.");
                return false;
            }
            
            Console.WriteLine("Connected to Voicemeeter");
            return _isConnected;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to connect to Voicemeeter: {ex.Message}");
            return false;
        }
    }

    public void SyncAllValues()
    {
        if (!_isConnected) return;

        try
        {
            int[] outputs = [_config.Output1, _config.Output2, _config.Output3];
            Console.WriteLine("Syncing Voicemeeter values...");
            foreach (var output in outputs)
            {
                SyncStripValues(output);
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error syncing values: {ex.Message}");
        }
    }
    
    private static void WaitDirty()
    {
        const int maxDelay = 2000;
        var delay = 10;
        
        while (VBVMR_IsParametersDirty() == 0)
        {
            if (delay >= maxDelay)
            {
                Console.WriteLine("Timeout waiting for Voicemeeter parameters to become dirty.");
                break;
            }
            Thread.Sleep(delay);
            delay *= 2;
        }
    }
    
    private static void WaitNotDirty()
    {
        const int maxDelay = 4000;
        var delay = 10;
        
        while (VBVMR_IsParametersDirty() != 0)
        {
            if (delay >= maxDelay)
            {
                Console.WriteLine("Timeout waiting for Voicemeeter parameters to become clean.");
                break;
            }
            Thread.Sleep(delay);
            delay *= 2;
        }
    }

    private void SyncStripValues(int strip)
    {
        try
        {
            WaitDirty();
            WaitNotDirty();
            var gainParam = $"Strip[{strip}].Gain";
            float gainValue = 0;
            if (VBVMR_GetParameterFloat(gainParam, ref gainValue) == 0)
            {
                _cachedGainValues[strip] = gainValue;
            }

            var muteParam = $"Strip[{strip}].Mute";
            float muteValue = 0;
            if (VBVMR_GetParameterFloat(muteParam, ref muteValue) == 0)
            {
                _cachedMuteValues[strip] = muteValue != 0;
            }
            Console.WriteLine($"Synced strip {strip}: Gain={gainValue:F1} dB, Mute={(muteValue != 0 ? "ON" : "OFF")}");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error syncing strip {strip}: {ex.Message}");
        }
    }

    public void QueueVolumeUp(int strip)
    {
        lock (_queueLock)
        {
            _actionQueue.Enqueue(new VolumeAction
            {
                Type = VolumeAction.ActionType.VolumeUp,
                Strip = strip
            });
        }
    }

    public void QueueVolumeDown(int strip)
    {
        lock (_queueLock)
        {
            _actionQueue.Enqueue(new VolumeAction
            {
                Type = VolumeAction.ActionType.VolumeDown,
                Strip = strip
            });
        }
    }

    public void QueueVolumeMute(int strip)
    {
        lock (_queueLock)
        {
            _actionQueue.Enqueue(new VolumeAction
            {
                Type = VolumeAction.ActionType.Mute,
                Strip = strip
            });
        }
    }

    public void QueueMediaAction(VolumeAction.ActionType action)
    {
        lock (_queueLock)
        {
            _actionQueue.Enqueue(new VolumeAction
            {
                Type = action,
                Strip = 0
            });
        }
    }

    private async Task ProcessActionsBackground()
    {
        while (!_cancellationTokenSource.Token.IsCancellationRequested)
        {
            try
            {
                var actionsToProcess = new List<VolumeAction>();

                lock (_queueLock)
                {
                    while (_actionQueue.Count > 0)
                    {
                        actionsToProcess.Add(_actionQueue.Dequeue());
                    }
                }

                foreach (var action in actionsToProcess)
                {
                    ProcessAction(action);
                }

                await Task.Delay(1, _cancellationTokenSource.Token); // 1ms delay
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error in background processing: {ex.Message}");
                await Task.Delay(10, _cancellationTokenSource.Token);
            }
        }
    }

    private void ProcessAction(VolumeAction action)
    {
        try
        {
            switch (action.Type)
            {
                case VolumeAction.ActionType.VolumeUp:
                    VolumeUp(action.Strip);
                    break;
                case VolumeAction.ActionType.VolumeDown:
                    VolumeDown(action.Strip);
                    break;
                case VolumeAction.ActionType.Mute:
                    VolumeMute(action.Strip);
                    break;
                case VolumeAction.ActionType.MediaNext:
                    WinAPI.keybd_event(WinAPI.VK_MEDIA_NEXT_TRACK, 0, WinAPI.KEYEVENTF_EXTENTEDKEY, IntPtr.Zero);
                    break;
                case VolumeAction.ActionType.MediaPrev:
                    WinAPI.keybd_event(WinAPI.VK_MEDIA_PREV_TRACK, 0, WinAPI.KEYEVENTF_EXTENTEDKEY, IntPtr.Zero);
                    break;
                case VolumeAction.ActionType.MediaPlayPause:
                    WinAPI.keybd_event(WinAPI.VK_MEDIA_PLAY_PAUSE, 0, WinAPI.KEYEVENTF_EXTENTEDKEY, IntPtr.Zero);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(action), action.Type, "Unknown action type");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error processing action {action.Type}: {ex.Message}");
        }
    }
    
    private void VolumeMute(int strip)
    {
        if (!_isConnected) return;

        try
        {
            var newMute = !_cachedMuteValues.GetValueOrDefault(strip, false);

            if (VBVMR_SetParameterFloat($"Strip[{strip}].Mute", newMute ? 1.0f : 0.0f) != 0)
            {
                throw new COMException("Unknown communication error.");
            }
            
            _cachedMuteValues[strip] = newMute;
            Console.WriteLine($"Strip {strip} mute: {(newMute ? "ON" : "OFF")}");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error toggling mute: {ex.Message}");
        }
    }

    private void VolumeSet(int strip, float newGain)
    {
        if (!_isConnected) return;

        try
        {
            newGain = Math.Max(-60.0f, Math.Min(12.0f, newGain));

            if (VBVMR_SetParameterFloat($"Strip[{strip}].Gain", newGain) != 0)
            {
                throw new COMException("Unknown communication error.");
            }
            
            _cachedGainValues[strip] = newGain;
            Console.WriteLine($"Strip {strip} volume set to: {newGain:F1} dB");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error setting volume: {ex.Message}");
        }
    }
    
    private void VolumeUp(int strip)
    {
        if (!_isConnected) return;

        var newGain = _cachedGainValues.GetValueOrDefault(strip, 0) + _config.VolumeChangeAmount;
        VolumeSet(strip, newGain);
    }

    private void VolumeDown(int strip)
    {
        if (!_isConnected) return;

        var newGain = _cachedGainValues.GetValueOrDefault(strip, 0) - _config.VolumeChangeAmount;
        VolumeSet(strip, newGain);
    }

    public void Restart()
    {
        if (!_isConnected) return;

        try
        {
            if (VBVMR_SetParameterStringA("Command.Restart", "1") != 0)
            {
                throw new COMException("Unknown communication error.");
            }
            
            Console.WriteLine("Voicemeeter restarted");
            
            _cachedGainValues.Clear();
            _cachedMuteValues.Clear();

            Task.Delay(3000).ContinueWith(t => {
                Connect();
            });
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error restarting Voicemeeter: {ex.Message}");
        }
    }

    public void LoadProfile(string filename)
    {
        if (!_isConnected) return;

        try
        {
            var fullPath = Path.Combine(Application.StartupPath, filename);
            if (File.Exists(fullPath))
            {
                if (VBVMR_SetParameterStringA("Command.Load", fullPath) != 0)
                {
                    throw new COMException("Unknown communication error.");
                }
                
                Console.WriteLine($"Loaded profile: {filename}");

                _cachedGainValues.Clear();
                _cachedMuteValues.Clear();

                SyncAllValues();
            }
            else
            {
                Console.WriteLine($"Profile file not found: {fullPath}");
                SyncAllValues();
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error loading profile: {ex.Message}");
        }
    }

    public void Disconnect()
    {
        _cancellationTokenSource.Cancel();
        _processingTask?.Wait(1000);

        if (!_isConnected) return;
        
        _ = VBVMR_Logout();
        _isConnected = false;
        _cachedGainValues.Clear();
        _cachedMuteValues.Clear();
    }
}

public struct VolumeAction
{
    public enum ActionType { VolumeUp, VolumeDown, Mute, MediaNext, MediaPrev, MediaPlayPause }
    public ActionType Type;
    public int Strip;
}