#!/usr/bin/env python

import sys
import argparse
import socket
import re
import requests
import json
import collections


version = "1.0"

# python 2 compatibility
try:
    input = raw_input
except NameError:
    pass


def parse_arguments():
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--ip", type=str, help="the ip address of the TV", required=False)
    parser.add_argument("-c", "--command", type=str, help="the command to send to the TV", required=False)
    args = parser.parse_args()
    return args


class BraviaConsole:
    def __init__(self):
        self.psk = "0000"
        self.ip = None
        self.sys_info = {}
        self.commands = {}
        self.model = "Bravia"

    def print_status(self, message):
        print(("[*] ") + (str(message)))

    def print_info(self, message):
        print(("[-] ") + (str(message)))

    def print_warning(self, message):
        print(("[!] ") + (str(message)))

    def print_error(self, message):
        print(("[!] ") + (str(message)))

    def exit_braviaremote(self):
        sys.exit()

    def show_commands(self):
        command_list = ""
        for command in self.commands:
            command_list += command + ", "
        if len(command_list) > 0:
            command_list = command_list[:-2]
        self.print_info(command_list)

    def send_command_to_tv(self, command):
        if command not in self.commands:
            return False
        ircc_code = self.commands[command]
        self.print_status("Sending command %s to TV" % command)
        body = "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:X_SendIRCC xmlns:u=\"urn:schemas-sony-com:service:IRCC:1\"><IRCCCode>" + ircc_code + "</IRCCCode></u:X_SendIRCC></s:Body></s:Envelope>"
        headers = {}
        headers['X-Auth-PSK'] = self.psk
        headers['Content-Type'] = "text/xml; charset=UTF-8"
        headers['SOAPACTION'] = "\"urn:schemas-sony-com:service:IRCC:1#X_SendIRCC\""
        try:
            response = requests.post('http://' + self.ip + '/sony/IRCC', headers=headers, data=body, timeout=5000)
            response.raise_for_status()
        except Exception as exception_instance:
            self.print_error("Exception: " + str(exception_instance))
        return True

    def set_ip_address(self, ip):
        match = re.search(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b", ip)
        if match:
            self.ip = match.group()
            self.print_info("IP address set to %s" % self.ip)
        else:
            self.print_error("Invalid IP address value pass in")

    def update_sys_info(self):
        self.print_info("Updating system info")
        if self.ip is None:
            self.print_error("TV was not found. Please run configure first")
            return
        result = self.send_info_request_to_tv("getSystemInformation")
        if result is not None:
            self.sys_info = result[0]
            self.model = self.sys_info["model"]
            self.print_status("TV model identified as %s" % self.model)

    def send_info_request_to_tv(self, command):
        body = {"method": command, "params": [], "id": 1, "version": "1.0"}
        json_body = json.dumps(body).encode('utf-8')
        headers = {}
        headers['X-Auth-PSK'] = self.psk
        try:
            response = requests.post('http://' + self.ip + '/sony/system', headers=headers, data=json_body, timeout=10)
            response.raise_for_status()
        except requests.exceptions.HTTPError as exception_instance:
            if response.status_code == 403:
                self.print_unauthorized_error()
            else:
                self.print_error("Exception: " + str(exception_instance))
            return None
        except Exception as exception_instance:
            self.print_error("Exception: " + str(exception_instance))
            return None
        else:
            return json.loads(response.content.decode('utf-8'))["result"]

    def update_commands(self):
        self.print_info("Updating commands")
        self.commands = {}
        result = self.send_info_request_to_tv("getRemoteControllerInfo")
        if result is not None:
            controller_commands = result[1]
            for command_data in controller_commands:
                self.commands[command_data.get('name').lower()] = command_data.get('value')
            self.commands = collections.OrderedDict(sorted(self.commands.items()))
            self.print_status("%d commands found" % len(self.commands))

    def auto_configure(self):
        self.print_info("Auto detecting settings")
        if self.ip is not None:
            self.update_sys_info()
            self.update_commands()
        else:
            self.print_error("Auto configuration failed, enter the command 'configure' to try again")

    def signal_handler(self, signal, frame):
        print("")
        self.exit_braviaremote()

    def execute_user_command(self, command):
        if command in self.commands:
            self.send_command_to_tv(command)
        elif command == "show commands":
            self.show_commands()
        else:
            self.print_warning("Command was not found, try help or ? for more information")

    def start(self, ip, command):
        if ip is not None:
            self.set_ip_address(ip)
        self.auto_configure()
        if command is not None:
            self.execute_user_command(command)
            self.exit_braviaremote()

        while 1:
            try:
                prompt = input(self.model + "> ")
            except EOFError:
                prompt = "quit"
                print("")

            self.execute_user_command(prompt)

def main():
    args = parse_arguments()
    console = BraviaConsole()
    console.start(args.ip, args.command)

if __name__ == "__main__":
    main()