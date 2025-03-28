package com.simongellis.vvb.menu

import android.app.Dialog
import android.hardware.input.InputManager
import android.os.Bundle
import android.view.InputDevice
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.core.content.ContextCompat
import androidx.fragment.app.DialogFragment
import androidx.fragment.app.viewModels
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import androidx.recyclerview.widget.SortedList
import androidx.recyclerview.widget.SortedListAdapterCallback
import com.simongellis.vvb.R

class AutoMapDialog : DialogFragment(), InputManager.InputDeviceListener {
    private val parentViewModel: ControllersViewModel by viewModels({ requireParentFragment() })
    private val _inputManager by lazy {
        ContextCompat.getSystemService(requireContext(), InputManager::class.java)!!
    }
    private val _adapter by lazy {
        DeviceListAdapter(
            requireContext().getString(R.string.controller_menu_no_controllers_detected),
            this::onDeviceChosen
        )
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        val context = requireContext()
        return AlertDialog.Builder(context)
            .setTitle(R.string.controller_menu_choose_controller)
            .setView(RecyclerView(context).apply {
                layoutManager = LinearLayoutManager(context)
                adapter = _adapter
            })
            .setNegativeButton(R.string.controller_menu_cancel) { _, _ ->
                dismiss()
            }
            .create()
    }

    override fun onStart() {
        super.onStart()
        _inputManager.registerInputDeviceListener(this, null)
        for (deviceId in _inputManager.inputDeviceIds) {
            onInputDeviceChanged(deviceId)
        }
    }

    override fun onStop() {
        super.onStop()
        _inputManager.unregisterInputDeviceListener(this)
    }

    private fun onDeviceChosen(deviceId: Int) {
        val device = _inputManager.getInputDevice(deviceId) ?: return
        parentViewModel.performAutoMap(device)
        dismiss()
    }

    override fun onInputDeviceAdded(deviceId: Int) {
        val device = _inputManager.getInputDevice(deviceId) ?: return
        if (parentViewModel.isMappable(device)) {
            _adapter.addDevice(device)
        }
    }

    override fun onInputDeviceRemoved(deviceId: Int) {
        _adapter.removeDevice(deviceId)
    }

    override fun onInputDeviceChanged(deviceId: Int) {
        val device = _inputManager.getInputDevice(deviceId)
        if (device != null && parentViewModel.isMappable(device)) {
            _adapter.addDevice(device)
        } else {
            _adapter.removeDevice(deviceId)
        }
    }

    class DeviceListAdapter(emptyMessage: String, private val callback: (Int) -> Unit): RecyclerView.Adapter<DeviceListAdapter.ViewHolder>() {
        data class Device(val id: Int?, val name: String)
        private val empty = Device(null, emptyMessage)

        private val _deviceList: SortedList<Device>
        init {
            val adapterCallback = object : SortedListAdapterCallback<Device>(this) {
                override fun compare(o1: Device, o2: Device): Int {
                    return o1.name.compareTo(o2.name)
                }

                override fun areContentsTheSame(oldItem: Device, newItem: Device): Boolean {
                    return oldItem.name == newItem.name
                }

                override fun areItemsTheSame(item1: Device, item2: Device): Boolean {
                    return item1.id == item2.id
                }
            }
            _deviceList = SortedList(Device::class.java, adapterCallback)
            _deviceList.add(empty)
        }

        fun addDevice(device: InputDevice) {
            _deviceList.remove(empty)
            _deviceList.add(Device(device.id, device.name))
        }

        fun removeDevice(deviceId: Int) {
            for (i in 0 until _deviceList.size()) {
                if (_deviceList[i].id == deviceId) {
                    _deviceList.removeItemAt(i)
                    break
                }
            }
            if (_deviceList.size() == 0) {
                _deviceList.add(empty)
            }
        }

        class ViewHolder(val view: TextView, callback: (Int) -> Unit): RecyclerView.ViewHolder(view) {
            var id: Int? = null
            init {
                view.setOnClickListener {
                    id?.also { callback(it) }
                }
            }
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
            val view = LayoutInflater.from(parent.context).inflate(android.R.layout.simple_list_item_1, parent, false)
            return ViewHolder(view as TextView, callback)
        }

        override fun onBindViewHolder(holder: ViewHolder, position: Int) {
            val device = _deviceList[position]
            holder.id = device.id
            holder.view.text = device.name
        }

        override fun getItemCount(): Int {
            return _deviceList.size()
        }
    }
}