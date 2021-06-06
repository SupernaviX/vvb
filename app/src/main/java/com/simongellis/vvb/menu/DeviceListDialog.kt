package com.simongellis.vvb.menu

import android.content.Context
import android.hardware.input.InputManager
import android.os.Bundle
import android.view.InputDevice
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.core.content.ContextCompat
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import androidx.recyclerview.widget.SortedList
import androidx.recyclerview.widget.SortedListAdapterCallback
import com.simongellis.vvb.R

class DeviceListDialog(context: Context) : AlertDialog(context), InputManager.InputDeviceListener {
    private val _inputManager = ContextCompat.getSystemService(context, InputManager::class.java)!!
    private val _adapter = DeviceListAdapter(
        context.getString(R.string.controller_menu_no_controllers_detected),
        this::onDeviceChosen)
    private var _deviceFilter: ((InputDevice) -> Boolean)? = null
    private var _onDeviceChosen: ((InputDevice) -> Unit)? = null

    init {
        _inputManager.registerInputDeviceListener(this, null)
        setView(RecyclerView(context).apply {
            layoutManager = LinearLayoutManager(context)
            adapter = _adapter
        })
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        setTitle(R.string.controller_menu_choose_controller)
        setButton(BUTTON_NEGATIVE, context.getText(R.string.controller_menu_cancel)) { _, _ ->
            dismiss()
        }
        super.onCreate(savedInstanceState)
    }

    override fun onStop() {
        super.onStop()
        _inputManager.unregisterInputDeviceListener(this)
    }

    fun setDeviceFilter(filter: (InputDevice) -> Boolean) {
        _deviceFilter = filter
        for (deviceId in _inputManager.inputDeviceIds) {
            onInputDeviceChanged(deviceId)
        }
    }

    fun setOnDeviceChosen(callback: (InputDevice) -> Unit) {
        _onDeviceChosen = callback
    }

    private fun onDeviceChosen(deviceId: Int) {
        val device = _inputManager.getInputDevice(deviceId)
        _onDeviceChosen?.invoke(device)
        dismiss()
    }

    override fun onInputDeviceAdded(deviceId: Int) {
        val device = _inputManager.getInputDevice(deviceId)
        if (_deviceFilter?.invoke(device) != false) {
            _adapter.addDevice(device)
        }
    }

    override fun onInputDeviceRemoved(deviceId: Int) {
        _adapter.removeDevice(deviceId)
    }

    override fun onInputDeviceChanged(deviceId: Int) {
        val device = _inputManager.getInputDevice(deviceId)
        if (_deviceFilter?.invoke(device) != false) {
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