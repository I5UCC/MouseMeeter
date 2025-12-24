using System;
using System.IO;

public static class FileLogger
{
    private static string LogFilePath => "log.txt";
    private static readonly Lock LockObj = new();
    
    public static void ClearLog()
    {
        lock (LockObj)
        {
            if (File.Exists(LogFilePath))
            {
                File.Delete(LogFilePath);
            }
        }
    }

    public static void Log(string message)
    {
        var logEntry = $"{DateTime.UtcNow:yyyy-MM-dd HH:mm:ss.fff} {message}";
        lock (LockObj)
        {
            File.AppendAllText(LogFilePath, logEntry + Environment.NewLine);
        }
    }
}