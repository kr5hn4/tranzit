import type { DeviceInfo } from "$lib/types/deviceInfo";

type FileInfo = { name: string; size: number };

export type Device = {
  name: string;
  ip: string;
  port: number;
  hostname: string;
  service_type: string;
  os: string;
  id: string;
};

export type SelectedFiles = {
  file_uuid: string;
  file_path: string;
  name: string;
  size: number;
  preview_base64?: string | null;
  mime_type: string;
  progress?: number;
};

type FileTransferRequestQueue = {
  id: string;
  data: {
    files_info: FileInfo[];
    device_info: DeviceInfo;
    receiverInfo: string;
  };
} | null;

export const store = $state<{
  isFocused: boolean; // tracks whether main app window is focused or not
  areDevicesRefreshing: boolean;
  devices: Device[]; // list of discovered devices on the network
  showFileTransferRequestPopup: boolean;
  showTransferProgressPopup: boolean;
  showPopup: boolean;
  popupMessage: string; // popup message for the generic popup
  fileTransferRequestQueue: FileTransferRequestQueue;
  waitingToAcceptTransferRequest: boolean;
  selectedFiles: SelectedFiles[];
  deviceInfo: DeviceInfo;
}>({
  isFocused: false,
  areDevicesRefreshing: false,
  devices: [],
  showFileTransferRequestPopup: false,
  showTransferProgressPopup: false,
  showPopup: false,
  popupMessage: "",
  fileTransferRequestQueue: null,
  waitingToAcceptTransferRequest: false,
  selectedFiles: [],
  deviceInfo: { hostname: "", os_type: "", id: "" },
});
