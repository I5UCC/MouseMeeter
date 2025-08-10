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
    public static extern int VBVMR_Login();

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern int VBVMR_Logout();

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern int VBVMR_GetParameterFloat([MarshalAs(UnmanagedType.LPStr)] string szParamName, ref float pValue);

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern int VBVMR_SetParameterFloat([MarshalAs(UnmanagedType.LPStr)] string szParamName, float Value);

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern int VBVMR_SetParameterStringA([MarshalAs(UnmanagedType.LPStr)] string szParamName, [MarshalAs(UnmanagedType.LPStr)] string szString);

    [DllImport("VoicemeeterRemote64.dll", CallingConvention = CallingConvention.Cdecl)]
    public static extern int VBVMR_IsParametersDirty();

    private MousemeeterConfig config;
    private bool isConnected = false;

    // Cached volume values
    private Dictionary<int, float> cachedGainValues = new Dictionary<int, float>();
    private Dictionary<int, bool> cachedMuteValues = new Dictionary<int, bool>();
    private DateTime lastSync = DateTime.MinValue;
    private readonly TimeSpan syncInterval = TimeSpan.FromSeconds(1);

    // Background processing
    private readonly Queue<VolumeAction> actionQueue = new Queue<VolumeAction>();
    private readonly object queueLock = new object();
    private readonly CancellationTokenSource cancellationTokenSource = new CancellationTokenSource();
    private Task processingTask;

    public VoicemeeterController(MousemeeterConfig config)
    {
        this.config = config;
        Connect();

        // Start background processing task
        processingTask = Task.Run(ProcessActionsBackground, cancellationTokenSource.Token);
    }

    public bool Connect()
    {
        try
        {
            int result = VBVMR_Login();
            isConnected = (result == 0 || result == 1);
            if (isConnected)
            {
                Console.WriteLine("Connected to Voicemeeter");
                SyncAllValues();
            }
            return isConnected;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to connect to Voicemeeter: {ex.Message}");
            return false;
        }
    }

    private void SyncAllValues()
    {
        if (!isConnected) return;

        try
        {
            int[] outputs = { config.Output1, config.Output2, config.Output3 };
            foreach (int output in outputs)
            {
                SyncStripValues(output);
            }
            lastSync = DateTime.Now;
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error syncing values: {ex.Message}");
        }
    }

    private void SyncStripValues(int strip)
    {
        try
        {
            string gainParam = $"Strip[{strip}].Gain";
            float gainValue = 0;
            if (VBVMR_GetParameterFloat(gainParam, ref gainValue) == 0)
            {
                cachedGainValues[strip] = gainValue;
            }

            string muteParam = $"Strip[{strip}].Mute";
            float muteValue = 0;
            if (VBVMR_GetParameterFloat(muteParam, ref muteValue) == 0)
            {
                cachedMuteValues[strip] = muteValue != 0;
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error syncing strip {strip}: {ex.Message}");
        }
    }

    public void QueueVolumeUp(int strip)
    {
        lock (queueLock)
        {
            actionQueue.Enqueue(new VolumeAction
            {
                Type = VolumeAction.ActionType.VolumeUp,
                Strip = strip,
                Timestamp = DateTime.Now
            });
        }
    }

    public void QueueVolumeDown(int strip)
    {
        lock (queueLock)
        {
            actionQueue.Enqueue(new VolumeAction
            {
                Type = VolumeAction.ActionType.VolumeDown,
                Strip = strip,
                Timestamp = DateTime.Now
            });
        }
    }

    public void QueueVolumeMute(int strip)
    {
        lock (queueLock)
        {
            actionQueue.Enqueue(new VolumeAction
            {
                Type = VolumeAction.ActionType.Mute,
                Strip = strip,
                Timestamp = DateTime.Now
            });
        }
    }

    public void QueueMediaAction(VolumeAction.ActionType action)
    {
        lock (queueLock)
        {
            actionQueue.Enqueue(new VolumeAction
            {
                Type = action,
                Strip = 0,
                Timestamp = DateTime.Now
            });
        }
    }

    private async Task ProcessActionsBackground()
    {
        while (!cancellationTokenSource.Token.IsCancellationRequested)
        {
            try
            {
                var actionsToProcess = new List<VolumeAction>();

                lock (queueLock)
                {
                    while (actionQueue.Count > 0)
                    {
                        actionsToProcess.Add(actionQueue.Dequeue());
                    }
                }

                foreach (var action in actionsToProcess)
                {
                    ProcessAction(action);
                }

                await Task.Delay(1, cancellationTokenSource.Token); // 1ms delay
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Error in background processing: {ex.Message}");
                await Task.Delay(10, cancellationTokenSource.Token);
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
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error processing action {action.Type}: {ex.Message}");
        }
    }

    private void VolumeUp(int strip)
    {
        if (!isConnected) return;

        try
        {
            if (!cachedGainValues.ContainsKey(strip))
            {
                SyncStripValues(strip);
            }

            float newGain = cachedGainValues.GetValueOrDefault(strip, 0) + config.VolumeChangeAmount;
            newGain = Math.Max(-60.0f, Math.Min(12.0f, newGain));

            string paramName = $"Strip[{strip}].Gain";
            if (VBVMR_SetParameterFloat(paramName, newGain) == 0)
            {
                cachedGainValues[strip] = newGain;
                Console.WriteLine($"Strip {strip} volume: {newGain:F1} dB");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error increasing volume: {ex.Message}");
        }
    }

    private void VolumeDown(int strip)
    {
        if (!isConnected) return;

        try
        {
            if (!cachedGainValues.ContainsKey(strip))
            {
                SyncStripValues(strip);
            }

            float newGain = cachedGainValues.GetValueOrDefault(strip, 0) - config.VolumeChangeAmount;
            newGain = Math.Max(-60.0f, Math.Min(12.0f, newGain));

            string paramName = $"Strip[{strip}].Gain";
            if (VBVMR_SetParameterFloat(paramName, newGain) == 0)
            {
                cachedGainValues[strip] = newGain;
                Console.WriteLine($"Strip {strip} volume: {newGain:F1} dB");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error decreasing volume: {ex.Message}");
        }
    }

    private void VolumeMute(int strip)
    {
        if (!isConnected) return;

        try
        {
            if (!cachedMuteValues.ContainsKey(strip))
            {
                SyncStripValues(strip);
            }

            bool newMute = !cachedMuteValues.GetValueOrDefault(strip, false);
            string paramName = $"Strip[{strip}].Mute";

            if (VBVMR_SetParameterFloat(paramName, newMute ? 1.0f : 0.0f) == 0)
            {
                cachedMuteValues[strip] = newMute;
                Console.WriteLine($"Strip {strip} mute: {(newMute ? "ON" : "OFF")}");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error toggling mute: {ex.Message}");
        }
    }

    public void Restart()
    {
        if (!isConnected) return;

        try
        {
            VBVMR_SetParameterStringA("Command.Restart", "1");
            cachedGainValues.Clear();
            cachedMuteValues.Clear();

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
        if (!isConnected) return;

        try
        {
            string fullPath = Path.Combine(Application.StartupPath, filename);
            if (File.Exists(fullPath))
            {
                VBVMR_SetParameterStringA("Command.Load", fullPath);
                Console.WriteLine($"Loaded profile: {filename}");

                cachedGainValues.Clear();
                cachedMuteValues.Clear();

                Task.Delay(1000).ContinueWith(t => {
                    SyncAllValues();
                });
            }
            else
            {
                Console.WriteLine($"Profile file not found: {fullPath}");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error loading profile: {ex.Message}");
        }
    }

    public void Disconnect()
    {
        cancellationTokenSource.Cancel();
        processingTask?.Wait(1000);

        if (isConnected)
        {
            VBVMR_Logout();
            isConnected = false;
            cachedGainValues.Clear();
            cachedMuteValues.Clear();
        }
    }
}
