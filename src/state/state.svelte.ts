import type { SysInfo } from "$lib/types/sysInfo";

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

type FileInfo = { name: string; size: number };

type FileTransferRequestQueue = {
  id: string;
  data: {
    files_info: FileInfo[];
    sys_info: SysInfo;
    receiverInfo: string;
  };
} | null;

export const store = $state<{
  isFocused: boolean; // tracks whether main app window is focused or not
  areDevicesRefreshing: boolean;
  devices: Device[]; // list of discovered devices on the network
  showFileTransferRequestPopup: boolean;
  showTransferProgressPopup: boolean;
  showGenericPopup: boolean;
  genericPopupMessage: string; // popup message for the generic popup
  fileTransferRequestQueue: FileTransferRequestQueue;
  waitingToAcceptTransferRequest: boolean;
  selectedFiles: SelectedFiles[];
  sysInfo: SysInfo;
}>({
  isFocused: false,
  areDevicesRefreshing: false,
  devices: [],
  showFileTransferRequestPopup: false,
  showTransferProgressPopup: false,
  showGenericPopup: false,
  genericPopupMessage: "",
  fileTransferRequestQueue: null,
  waitingToAcceptTransferRequest: false,
  selectedFiles: [],
  sysInfo: { hostname: "", os_type: "", app_id: "" },
});
