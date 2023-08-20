#define PY_SSIZE_T_CLEAN
#include <Python.h>

static PyObject* hello_world(PyObject* self, PyObject* args) {
    /**
     * Parameters
     * ----------
     * self
     *  A pointer to a module object for functions or an object instance for methods.
     * args
     *  A pointer to a Python tuple object containing arguments to the function.
     * 
     * Returns
     * -------
     * 
    */
    return Py_BuildValue("s", "Hello, C++ Extension!");
}

static PyMethodDef methods[] = {
    {"hello", hello_world, METH_NOARGS, "Print a greeting from C++ extension"},
    {NULL, NULL, 0, NULL}
};

static struct PyModuleDef module = {
    PyModuleDef_HEAD_INIT,
    "hello",
    NULL,
    -1,
    methods
};

PyMODINIT_FUNC PyInit_hello(void) {
    return PyModule_Create(&module);
}
